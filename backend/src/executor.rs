use anyhow::{anyhow, Result};
use std::collections::{HashMap, VecDeque};
use tokio::sync::broadcast;

use crate::metrics::{self, Timer};
use crate::models::{
    FlowNode, Pipeline, PipelineStatus, ProcessRequest, ProcessResponse, WsEvent,
};
use crate::AppState;

/// Run one execution pass through the pipeline.
///
/// `trigger_data` is fed to every source node (no incoming edges).
/// Returns a map of node_id -> output values for the terminal nodes.
pub async fn execute_pipeline(
    state: &AppState,
    pipeline: &Pipeline,
    trigger_data: serde_json::Value,
) -> Result<HashMap<String, serde_json::Value>> {
    let ws_tx = &state.ws_tx;
    let pid = &pipeline.id;

    emit(ws_tx, WsEvent::pipeline_status(pid, &PipelineStatus::Running));
    emit(
        ws_tx,
        WsEvent::execution_log(pid, "Execution started"),
    );

    // Build adjacency and reverse-adjacency structures
    let node_map: HashMap<&str, &FlowNode> = pipeline
        .nodes
        .iter()
        .map(|n| (n.id.as_str(), n))
        .collect();

    // incoming edge count per node
    let mut in_degree: HashMap<&str, usize> = node_map.keys().map(|k| (*k, 0)).collect();
    // node -> list of (target_node, target_port)
    let mut out_edges: HashMap<&str, Vec<(&str, &str, &str)>> = HashMap::new();

    for edge in &pipeline.edges {
        *in_degree.entry(&edge.target).or_insert(0) += 1;
        out_edges
            .entry(&edge.source)
            .or_default()
            .push((&edge.target, &edge.source_handle, &edge.target_handle));
    }

    // node_id -> port_name -> accumulated input data
    let mut input_buffers: HashMap<&str, HashMap<String, serde_json::Value>> = HashMap::new();
    // track how many inputs each node has satisfied
    let mut satisfied: HashMap<&str, usize> = HashMap::new();

    // Seed source nodes (in_degree == 0) with trigger_data
    let mut queue: VecDeque<&str> = VecDeque::new();
    for (nid, deg) in &in_degree {
        if *deg == 0 {
            // Source node: put trigger data on all its input ports (or use "input")
            let node = node_map[*nid];
            let port_id = node
                .input_ports
                .first()
                .map(|p| p.id.clone())
                .unwrap_or_else(|| "input".to_string());
            input_buffers
                .entry(nid)
                .or_default()
                .insert(port_id, trigger_data.clone());
            queue.push_back(nid);
        }
    }

    let mut final_outputs: HashMap<String, serde_json::Value> = HashMap::new();

    while let Some(nid) = queue.pop_front() {
        let node = node_map[nid];
        let inputs = input_buffers.remove(nid).unwrap_or_default();

        emit(ws_tx, WsEvent::node_status(pid, nid, "processing"));
        emit(
            ws_tx,
            WsEvent::execution_log(pid, &format!("Processing node '{}'", node.label)),
        );

        let outputs = call_node(state, node, inputs, &node.config).await;

        match outputs {
            Err(e) => {
                let msg = format!("Node '{}' error: {}", node.label, e);
                emit(ws_tx, WsEvent::execution_log(pid, &msg));
                emit(ws_tx, WsEvent::node_status(pid, nid, "error"));
                continue;
            }
            Ok(outputs) => {
                emit(
                    ws_tx,
                    WsEvent::node_output(pid, nid, &serde_json::json!(outputs)),
                );
                emit(ws_tx, WsEvent::node_status(pid, nid, "done"));

                // Route outputs to downstream nodes
                let downstream = out_edges.get(nid).cloned().unwrap_or_default();
                if downstream.is_empty() {
                    // Terminal node — record output
                    for (port, val) in &outputs {
                        final_outputs.insert(format!("{nid}:{port}"), val.clone());
                    }
                }

                for (target_id, src_port, tgt_port) in downstream {
                    if let Some(val) = outputs.get(src_port) {
                        input_buffers
                            .entry(target_id)
                            .or_default()
                            .insert(tgt_port.to_string(), val.clone());

                        let sat = satisfied.entry(target_id).or_insert(0);
                        *sat += 1;

                        // Ready when all incoming edges for this node are satisfied
                        let required = in_degree[target_id];
                        if *sat >= required {
                            queue.push_back(target_id);
                        }
                    }
                }
            }
        }
    }

    emit(
        ws_tx,
        WsEvent::execution_log(pid, "Execution complete"),
    );

    Ok(final_outputs)
}

async fn call_node(
    state: &AppState,
    node: &FlowNode,
    inputs: HashMap<String, serde_json::Value>,
    config: &HashMap<String, serde_json::Value>,
) -> Result<HashMap<String, serde_json::Value>> {
    let host_port = state
        .docker
        .host_port_for(&node.id)
        .ok_or_else(|| anyhow!("No running container for node '{}' — deploy first", node.id))?;

    let url = format!("http://127.0.0.1:{host_port}/process");

    let req = ProcessRequest {
        inputs,
        config: config.clone(),
    };

    // Measure bytes in
    let req_bytes = serde_json::to_string(&req).map(|s| s.len()).unwrap_or(0);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let timer = Timer::start();

    let resp = client
        .post(&url)
        .json(&req)
        .send()
        .await
        .map_err(|e| anyhow!("HTTP call to container failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        metrics::record(&state.metrics, &node.id, req_bytes, 0, timer.elapsed_ms(), true);
        emit(&state.ws_tx, WsEvent::metrics_update(&node.id, &state.metrics.get(&node.id).map(|m| m.clone()).unwrap_or_default()));
        return Err(anyhow!("Container returned {status}: {body}"));
    }

    let result: ProcessResponse = resp
        .json()
        .await
        .map_err(|e| anyhow!("Invalid response from container: {e}"))?;

    // Record metrics
    let resp_bytes = serde_json::to_string(&result.outputs).map(|s| s.len()).unwrap_or(0);
    let latency = timer.elapsed_ms();
    metrics::record(&state.metrics, &node.id, req_bytes, resp_bytes, latency, false);
    emit(
        &state.ws_tx,
        WsEvent::metrics_update(
            &node.id,
            &state.metrics.get(&node.id).map(|m| m.clone()).unwrap_or_default(),
        ),
    );

    Ok(result.outputs)
}

fn emit(tx: &broadcast::Sender<String>, event: WsEvent) {
    if let Ok(json) = serde_json::to_string(&event) {
        let _ = tx.send(json);
    }
}
