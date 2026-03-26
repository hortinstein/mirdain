pub mod execution;
pub mod node_types;
pub mod pipelines;
pub mod ws;

use axum::{
    routing::{get, post, put},
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
        // WebSocket
        .route("/ws", get(ws::handler))
        .with_state(state)
}
