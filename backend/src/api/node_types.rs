use axum::{extract::State, Json};

use crate::models::NodeType;
use crate::state::AppState;

pub async fn list(State(state): State<AppState>) -> Json<Vec<NodeType>> {
    Json(state.node_types.read().await.clone())
}
