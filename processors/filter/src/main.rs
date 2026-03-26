/// Mirdain Filter Processor
///
/// Routes input data to "matched" or "unmatched" output based on a substring
/// pattern test against a named field in the JSON input.
///
/// Config:
///   FIELD   - the JSON field to test (default: "value")
///   PATTERN - the substring to look for (default: "")
use axum::{routing::get, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize)]
struct ProcessRequest {
    inputs: HashMap<String, serde_json::Value>,
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
        name: "Filter",
        description: "Routes data to 'matched' or 'unmatched' based on a field pattern",
        version: "1.0.0",
        inputs: vec![Port { id: "input", label: "Input" }],
        outputs: vec![
            Port { id: "matched", label: "Matched" },
            Port { id: "unmatched", label: "Unmatched" },
        ],
    })
}

async fn process(Json(req): Json<ProcessRequest>) -> Json<ProcessResponse> {
    let field = config_str(&req.config, "field")
        .or_else(|| std::env::var("FIELD").ok())
        .unwrap_or_else(|| "value".to_string());

    let pattern = config_str(&req.config, "pattern")
        .or_else(|| std::env::var("PATTERN").ok())
        .unwrap_or_default();

    let input = req.inputs.get("input").cloned().unwrap_or(serde_json::Value::Null);

    let test_val = match &input {
        serde_json::Value::Object(map) => map
            .get(&field)
            .map(|v| v.to_string())
            .unwrap_or_default(),
        other => other.to_string(),
    };

    let matches = pattern.is_empty() || test_val.contains(&pattern);

    let mut outputs = HashMap::new();
    if matches {
        outputs.insert("matched".to_string(), input);
    } else {
        outputs.insert("unmatched".to_string(), input);
    }
    Json(ProcessResponse { outputs })
}

fn config_str(config: &HashMap<String, serde_json::Value>, key: &str) -> Option<String> {
    config.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
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
    println!("Filter processor listening on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
