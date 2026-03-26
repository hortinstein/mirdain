pub mod codegen;
pub mod execution;
pub mod metrics;
pub mod node_types;
pub mod pipelines;
pub mod ws;

use axum::{
    routing::{get, post},
    Router,
};

use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        // Pipeline CRUD
        .route("/api/pipelines", get(pipelines::list).post(pipelines::create))
        .route(
            "/api/pipelines/:id",
            get(pipelines::get_one)
                .put(pipelines::update)
                .delete(pipelines::remove),
        )
        // Execution
        .route("/api/pipelines/:id/deploy", post(execution::deploy))
        .route("/api/pipelines/:id/stop", post(execution::stop))
        .route("/api/pipelines/:id/trigger", post(execution::trigger))
        // Node type registry
        .route("/api/node-types", get(node_types::list))
        // AI code generation
        .route("/api/node-types/generate", post(codegen::generate))
        .route("/api/node-types/:id/rebuild", post(codegen::rebuild))
        .route("/api/node-types/:id/source", get(codegen::get_source))
        // Metrics
        .route("/api/metrics", get(metrics::all))
        // WebSocket
        .route("/ws", get(ws::handler))
        .with_state(state)
}
