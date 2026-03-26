use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub id: String,
    pub name: String,
    pub nodes: Vec<FlowNode>,
    pub edges: Vec<FlowEdge>,
    #[serde(default)]
    pub status: PipelineStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PipelineStatus {
    #[default]
    Stopped,
    Running,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowNode {
    pub id: String,
    pub label: String,
    /// Docker image for this node
    pub image: String,
    pub position: Position,
    /// Runtime config passed to the container as env vars / process config
    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,
    /// Input port definitions (populated from node type or manifest)
    #[serde(default)]
    pub input_ports: Vec<Port>,
    /// Output port definitions
    #[serde(default)]
    pub output_ports: Vec<Port>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowEdge {
    pub id: String,
    pub source: String,
    #[serde(rename = "sourceHandle")]
    pub source_handle: String,
    pub target: String,
    #[serde(rename = "targetHandle")]
    pub target_handle: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

/// Returned by GET /api/node-types — catalogue of known images
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeType {
    pub id: String,
    pub name: String,
    pub image: String,
    pub description: String,
    pub input_ports: Vec<Port>,
    pub output_ports: Vec<Port>,
    #[serde(default)]
    pub default_config: HashMap<String, serde_json::Value>,
    /// Prompt used to AI-generate this node (None for built-ins)
    #[serde(default)]
    pub ai_prompt: Option<String>,
    /// Generated Rust source (None for built-ins)
    #[serde(default)]
    pub source_code: Option<String>,
    /// Build state for AI-generated nodes
    #[serde(default)]
    pub build_status: BuildStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BuildStatus {
    #[default]
    Ready,
    Building,
    Error,
}

// ── Container protocol ───────────────────────────────────────────────────────

/// Sent to POST /process on each container
#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessRequest {
    pub inputs: HashMap<String, serde_json::Value>,
    pub config: HashMap<String, serde_json::Value>,
}

/// Returned by POST /process
#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessResponse {
    pub outputs: HashMap<String, serde_json::Value>,
}

/// Returned by GET /manifest
#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub name: String,
    pub description: String,
    pub version: String,
    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
}

// ── WebSocket events sent to the browser ────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(flatten)]
    pub payload: serde_json::Value,
}

impl WsEvent {
    pub fn pipeline_status(pipeline_id: &str, status: &PipelineStatus) -> Self {
        Self {
            event_type: "pipeline_status".into(),
            payload: serde_json::json!({
                "pipelineId": pipeline_id,
                "status": status,
            }),
        }
    }

    pub fn node_status(pipeline_id: &str, node_id: &str, status: &str) -> Self {
        Self {
            event_type: "node_status".into(),
            payload: serde_json::json!({
                "pipelineId": pipeline_id,
                "nodeId": node_id,
                "status": status,
            }),
        }
    }

    pub fn execution_log(pipeline_id: &str, message: &str) -> Self {
        Self {
            event_type: "execution_log".into(),
            payload: serde_json::json!({
                "pipelineId": pipeline_id,
                "message": message,
            }),
        }
    }

    pub fn node_output(
        pipeline_id: &str,
        node_id: &str,
        data: &serde_json::Value,
    ) -> Self {
        Self {
            event_type: "node_output".into(),
            payload: serde_json::json!({
                "pipelineId": pipeline_id,
                "nodeId": node_id,
                "data": data,
            }),
        }
    }

    pub fn metrics_update(node_id: &str, metrics: &crate::metrics::NodeMetrics) -> Self {
        Self {
            event_type: "metrics_update".into(),
            payload: serde_json::json!({
                "nodeId": node_id,
                "metrics": metrics,
            }),
        }
    }

    pub fn build_log(node_type_id: &str, message: &str) -> Self {
        Self {
            event_type: "build_log".into(),
            payload: serde_json::json!({
                "nodeTypeId": node_type_id,
                "message": message,
            }),
        }
    }

    pub fn build_complete(node_type_id: &str, image: &str) -> Self {
        Self {
            event_type: "build_complete".into(),
            payload: serde_json::json!({
                "nodeTypeId": node_type_id,
                "image": image,
                "status": "ready",
            }),
        }
    }

    pub fn build_error(node_type_id: &str, message: &str) -> Self {
        Self {
            event_type: "build_error".into(),
            payload: serde_json::json!({
                "nodeTypeId": node_type_id,
                "message": message,
            }),
        }
    }

    pub fn node_types_updated() -> Self {
        Self {
            event_type: "node_types_updated".into(),
            payload: serde_json::json!({}),
        }
    }
}
