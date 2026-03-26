use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::models::{FlowEdge, FlowNode, Pipeline, PipelineStatus};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CreatePipelineRequest {
    pub name: String,
    #[serde(default)]
    pub nodes: Vec<FlowNode>,
    #[serde(default)]
    pub edges: Vec<FlowEdge>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePipelineRequest {
    pub name: Option<String>,
    pub nodes: Option<Vec<FlowNode>>,
    pub edges: Option<Vec<FlowEdge>>,
}

pub async fn list(State(state): State<AppState>) -> Json<Vec<Pipeline>> {
    Json(state.store.list().await)
}

pub async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Pipeline>, StatusCode> {
    state
        .store
        .get(&id)
        .await
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

pub async fn create(
    State(state): State<AppState>,
    Json(body): Json<CreatePipelineRequest>,
) -> Result<(StatusCode, Json<Pipeline>), StatusCode> {
    let pipeline = Pipeline {
        id: Uuid::new_v4().to_string(),
        name: body.name,
        nodes: body.nodes,
        edges: body.edges,
        status: PipelineStatus::Stopped,
    };
    state
        .store
        .upsert(pipeline.clone())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(pipeline)))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdatePipelineRequest>,
) -> Result<Json<Pipeline>, StatusCode> {
    let mut pipeline = state
        .store
        .get(&id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;

    if let Some(name) = body.name {
        pipeline.name = name;
    }
    if let Some(nodes) = body.nodes {
        pipeline.nodes = nodes;
    }
    if let Some(edges) = body.edges {
        pipeline.edges = edges;
    }

    state
        .store
        .upsert(pipeline.clone())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(pipeline))
}

pub async fn remove(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> StatusCode {
    match state.store.delete(&id).await {
        Ok(true) => StatusCode::NO_CONTENT,
        Ok(false) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
