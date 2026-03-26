/// Mirdain Uppercase Processor
///
/// Converts the "input" field to uppercase and emits it as "output".
/// Handles strings, arrays of strings, or objects by stringifying.
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
        name: "Uppercase",
        description: "Converts input text to uppercase",
        version: "1.0.0",
        inputs: vec![Port { id: "input", label: "Input" }],
        outputs: vec![Port { id: "output", label: "Output" }],
    })
}

async fn process(Json(req): Json<ProcessRequest>) -> Json<ProcessResponse> {
    let input = req.inputs.get("input").cloned().unwrap_or(serde_json::Value::Null);
    let output = uppercase_value(input);
    let mut outputs = HashMap::new();
    outputs.insert("output".to_string(), output);
    Json(ProcessResponse { outputs })
}

fn uppercase_value(val: serde_json::Value) -> serde_json::Value {
    match val {
        serde_json::Value::String(s) => serde_json::Value::String(s.to_uppercase()),
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(uppercase_value).collect())
        }
        other => serde_json::Value::String(other.to_string().to_uppercase()),
    }
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
    println!("Uppercase processor listening on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
