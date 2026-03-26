mod ai_codegen;
mod api;
mod builder;
mod docker_mgr;
mod executor;
mod metrics;
mod models;
mod state;
mod store;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use axum::http::Method;
use tokio::sync::{broadcast, RwLock};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::docker_mgr::DockerManager;
use crate::models::{BuildStatus, NodeType, Port};
use crate::state::{AppState, AppStateInner};
use crate::store::PipelineStore;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("mirdain=info".parse()?))
        .init();

    let data_path = PathBuf::from(
        std::env::var("DATA_PATH").unwrap_or_else(|_| "data/pipelines.json".into()),
    );
    let processors_dir = PathBuf::from(
        std::env::var("PROCESSORS_DIR").unwrap_or_else(|_| "data/processors".into()),
    );
    tokio::fs::create_dir_all(&processors_dir).await?;

    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3001".into());
    let anthropic_api_key = std::env::var("ANTHROPIC_API_KEY").ok();

    if anthropic_api_key.is_none() {
        tracing::warn!("ANTHROPIC_API_KEY not set — AI node generation will be unavailable");
    }

    let store = PipelineStore::load(data_path).await?;

    let docker = DockerManager::new().unwrap_or_else(|e| {
        tracing::warn!("Docker unavailable ({e}); container operations will fail at runtime");
        DockerManager::new().expect("second DockerManager::new should not fail")
    });

    let (ws_tx, _) = broadcast::channel::<String>(512);

    // Load built-in node types + any previously generated custom types
    let mut node_types = built_in_node_types();
    node_types.extend(load_custom_node_types(&processors_dir).await);

    let state: AppState = Arc::new(AppStateInner {
        store,
        docker,
        ws_tx,
        node_types: RwLock::new(node_types),
        metrics: metrics::new_store(),
        anthropic_api_key,
        processors_dir,
    });

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any)
        .allow_origin(Any);

    let app = api::router(state).layer(cors);

    info!("Listening on {bind_addr}");
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn load_custom_node_types(processors_dir: &PathBuf) -> Vec<NodeType> {
    let registry_path = processors_dir.join("registry.json");
    match tokio::fs::read_to_string(&registry_path).await {
        Ok(raw) => serde_json::from_str::<Vec<NodeType>>(&raw).unwrap_or_default(),
        Err(_) => vec![],
    }
}

fn built_in_node_types() -> Vec<NodeType> {
    vec![
        NodeType {
            id: "mirdain/uppercase".into(),
            name: "Uppercase".into(),
            image: "mirdain/uppercase:latest".into(),
            description: "Converts string data to uppercase".into(),
            input_ports: vec![Port { id: "input".into(), label: "Input".into() }],
            output_ports: vec![Port { id: "output".into(), label: "Output".into() }],
            default_config: HashMap::new(),
            ai_prompt: None,
            source_code: None,
            build_status: BuildStatus::Ready,
        },
        NodeType {
            id: "mirdain/filter".into(),
            name: "Filter".into(),
            image: "mirdain/filter:latest".into(),
            description: "Filters records by a substring match on a field".into(),
            input_ports: vec![Port { id: "input".into(), label: "Input".into() }],
            output_ports: vec![
                Port { id: "matched".into(), label: "Matched".into() },
                Port { id: "unmatched".into(), label: "Unmatched".into() },
            ],
            default_config: {
                let mut m = HashMap::new();
                m.insert("field".into(), serde_json::json!("value"));
                m.insert("pattern".into(), serde_json::json!(""));
                m
            },
            ai_prompt: None,
            source_code: None,
            build_status: BuildStatus::Ready,
        },
        NodeType {
            id: "mirdain/merge".into(),
            name: "Merge".into(),
            image: "mirdain/merge:latest".into(),
            description: "Merges two inputs into a single JSON object".into(),
            input_ports: vec![
                Port { id: "left".into(), label: "Left".into() },
                Port { id: "right".into(), label: "Right".into() },
            ],
            output_ports: vec![Port { id: "output".into(), label: "Output".into() }],
            default_config: HashMap::new(),
            ai_prompt: None,
            source_code: None,
            build_status: BuildStatus::Ready,
        },
        NodeType {
            id: "custom".into(),
            name: "Custom Container".into(),
            image: "".into(),
            description: "Any Docker image implementing the mirdain processor protocol".into(),
            input_ports: vec![Port { id: "input".into(), label: "Input".into() }],
            output_ports: vec![Port { id: "output".into(), label: "Output".into() }],
            default_config: HashMap::new(),
            ai_prompt: None,
            source_code: None,
            build_status: BuildStatus::Ready,
        },
    ]
}
