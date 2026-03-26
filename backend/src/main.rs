mod api;
mod docker_mgr;
mod executor;
mod models;
mod state;
mod store;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use axum::http::Method;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::docker_mgr::DockerManager;
use crate::models::{NodeType, Port};
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
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3001".into());

    let store = PipelineStore::load(data_path).await?;

    let docker = DockerManager::new().unwrap_or_else(|e| {
        tracing::warn!("Docker unavailable ({e}); container operations will fail at runtime");
        // Create a dummy manager that will fail gracefully
        DockerManager::new().expect("should not panic on second try")
    });

    let (ws_tx, _) = broadcast::channel::<String>(256);

    let node_types = built_in_node_types();

    let state: AppState = Arc::new(AppStateInner {
        store,
        docker,
        ws_tx,
        node_types,
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

fn built_in_node_types() -> Vec<NodeType> {
    vec![
        NodeType {
            id: "mirdain/uppercase".into(),
            name: "Uppercase".into(),
            image: "mirdain/uppercase:latest".into(),
            description: "Converts string data to uppercase".into(),
            input_ports: vec![Port {
                id: "input".into(),
                label: "Input".into(),
            }],
            output_ports: vec![Port {
                id: "output".into(),
                label: "Output".into(),
            }],
            default_config: HashMap::new(),
        },
        NodeType {
            id: "mirdain/filter".into(),
            name: "Filter".into(),
            image: "mirdain/filter:latest".into(),
            description: "Filters records by a substring match on a field".into(),
            input_ports: vec![Port {
                id: "input".into(),
                label: "Input".into(),
            }],
            output_ports: vec![
                Port {
                    id: "matched".into(),
                    label: "Matched".into(),
                },
                Port {
                    id: "unmatched".into(),
                    label: "Unmatched".into(),
                },
            ],
            default_config: {
                let mut m = HashMap::new();
                m.insert("field".into(), serde_json::json!("value"));
                m.insert("pattern".into(), serde_json::json!(""));
                m
            },
        },
        NodeType {
            id: "mirdain/merge".into(),
            name: "Merge".into(),
            image: "mirdain/merge:latest".into(),
            description: "Merges two inputs into a single JSON object".into(),
            input_ports: vec![
                Port {
                    id: "left".into(),
                    label: "Left".into(),
                },
                Port {
                    id: "right".into(),
                    label: "Right".into(),
                },
            ],
            output_ports: vec![Port {
                id: "output".into(),
                label: "Output".into(),
            }],
            default_config: HashMap::new(),
        },
        NodeType {
            id: "custom".into(),
            name: "Custom Container".into(),
            image: "".into(),
            description: "Any Docker image implementing the mirdain processor protocol".into(),
            input_ports: vec![Port {
                id: "input".into(),
                label: "Input".into(),
            }],
            output_ports: vec![Port {
                id: "output".into(),
                label: "Output".into(),
            }],
            default_config: HashMap::new(),
        },
    ]
}
