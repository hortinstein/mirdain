/// Mirdain Merge Processor
///
/// Merges "left" and "right" inputs into a single JSON object:
///   {"left": <left_value>, "right": <right_value>}
///
/// This is a fan-in node — the orchestrator waits for both inputs before calling /process.
use axum::{routing::get, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize)]
struct ProcessRequest {
    inputs: HashMap<String, serde_json::Value>,
    #[allow(dead_code)]
    config: HashMap<String, serde_json::Value>,
}

#[derive(Serialize)]
struct ProcessResponse {
    outputs: HashMap<String, serde_json::Value>,
}

#[derive(Serialize)]
struct Manifest {
    name: &'static str,
    description: &'static str,
    version: &'static str,
    inputs: Vec<Port>,
    outputs: Vec<Port>,
}

#[derive(Serialize)]
struct Port {
    id: &'static str,
    label: &'static str,
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

async fn manifest() -> Json<Manifest> {
    Json(Manifest {
        name: "Merge",
        description: "Fan-in: merges left and right inputs into one object",
        version: "1.0.0",
        inputs: vec![
            Port { id: "left", label: "Left" },
            Port { id: "right", label: "Right" },
        ],
        outputs: vec![Port { id: "output", label: "Output" }],
    })
}

async fn process(Json(req): Json<ProcessRequest>) -> Json<ProcessResponse> {
    let left = req.inputs.get("left").cloned().unwrap_or(serde_json::Value::Null);
    let right = req.inputs.get("right").cloned().unwrap_or(serde_json::Value::Null);

    let merged = serde_json::json!({
        "left": left,
        "right": right,
    });

    let mut outputs = HashMap::new();
    outputs.insert("output".to_string(), merged);
    Json(ProcessResponse { outputs })
}

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let app = Router::new()
        .route("/health", get(health))
        .route("/manifest", get(manifest))
        .route("/process", post(process));

    let addr = format!("0.0.0.0:{port}");
    println!("Merge processor listening on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
