use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::executor::execute_pipeline;
use crate::models::{PipelineStatus, WsEvent};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct TriggerRequest {
    #[serde(default = "serde_json::Value::default")]
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ExecResult {
    pub outputs: serde_json::Value,
}

pub async fn deploy(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut pipeline = state
        .store
        .get(&id)
        .await
        .ok_or((StatusCode::NOT_FOUND, "Pipeline not found".into()))?;

    let network = state
        .docker
        .ensure_network(&id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    for node in &pipeline.nodes {
        emit(
            &state,
            WsEvent::node_status(&id, &node.id, "starting"),
        );
        let port = state
            .docker
            .start_node(&id, node, &network)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to start '{}': {}", node.label, e),
                )
            })?;
        emit(
            &state,
            WsEvent::execution_log(
                &id,
                &format!("Node '{}' running on host port {}", node.label, port),
            ),
        );
    }

    pipeline.status = PipelineStatus::Running;
    state
        .store
        .upsert(pipeline)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    emit(&state, WsEvent::pipeline_status(&id, &PipelineStatus::Running));

    Ok(Json(serde_json::json!({"status": "running"})))
}

pub async fn stop(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut pipeline = state
        .store
        .get(&id)
        .await
        .ok_or((StatusCode::NOT_FOUND, "Pipeline not found".into()))?;

    let node_ids: Vec<String> = pipeline.nodes.iter().map(|n| n.id.clone()).collect();
    state
        .docker
        .stop_all_nodes(&id, &node_ids)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    state
        .docker
        .remove_network(&id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    pipeline.status = PipelineStatus::Stopped;
    state
        .store
        .upsert(pipeline)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    emit(&state, WsEvent::pipeline_status(&id, &PipelineStatus::Stopped));

    Ok(Json(serde_json::json!({"status": "stopped"})))
}

pub async fn trigger(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<TriggerRequest>,
) -> Result<Json<ExecResult>, (StatusCode, String)> {
    let pipeline = state
        .store
        .get(&id)
        .await
        .ok_or((StatusCode::NOT_FOUND, "Pipeline not found".into()))?;

    let outputs = execute_pipeline(&state, &pipeline, body.data)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ExecResult {
        outputs: serde_json::json!(outputs),
    }))
}

fn emit(state: &AppState, event: WsEvent) {
    if let Ok(json) = serde_json::to_string(&event) {
        let _ = state.ws_tx.send(json);
    }
}
