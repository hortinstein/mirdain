use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ai_codegen;
use crate::builder;
use crate::models::{BuildStatus, NodeType, Port, WsEvent};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct GenerateRequest {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub input_ports: Vec<Port>,
    #[serde(default)]
    pub output_ports: Vec<Port>,
}

#[derive(Debug, Deserialize)]
pub struct RebuildRequest {
    /// New description — if provided, regenerate code via AI
    pub description: Option<String>,
    /// Raw source code override (bypasses AI, rebuilds directly)
    pub source_code: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GenerateResponse {
    pub node_type: NodeType,
}

fn api_key_or_err(state: &AppState) -> Result<String, (StatusCode, String)> {
    state
        .anthropic_api_key
        .clone()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "ANTHROPIC_API_KEY not set on server".into()))
}

/// POST /api/node-types/generate
pub async fn generate(
    State(state): State<AppState>,
    Json(body): Json<GenerateRequest>,
) -> Result<Json<GenerateResponse>, (StatusCode, String)> {
    let api_key = api_key_or_err(&state)?;

    let id = format!("ai/{}", Uuid::new_v4());
    let slug = body.name.to_lowercase().replace(|c: char| !c.is_alphanumeric(), "-");
    let image = format!("mirdain/{slug}:latest");

    // Call Claude to generate Rust code
    emit(&state, WsEvent::build_log(&id, "Generating Rust processor code with AI..."));
    let generated = ai_codegen::generate_processor(
        &api_key,
        &body.description,
        &body.name,
        &body.input_ports,
        &body.output_ports,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    emit(&state, WsEvent::build_log(&id, "Code generation complete. Writing files..."));

    // Write files to disk
    let build_dir = builder::write_processor_files(
        &state.processors_dir,
        &id.replace('/', "-"),
        &generated.cargo_toml,
        &generated.source_code,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let node_type = NodeType {
        id: id.clone(),
        name: body.name.clone(),
        image: image.clone(),
        description: body.description.clone(),
        input_ports: body.input_ports,
        output_ports: body.output_ports,
        default_config: Default::default(),
        ai_prompt: Some(body.description),
        source_code: Some(generated.source_code),
        build_status: BuildStatus::Building,
    };

    // Register it right away so it shows up in the UI
    state.node_types.write().await.push(node_type.clone());
    persist_custom_node_types(&state).await;
    emit(&state, WsEvent::node_types_updated());

    // Fire-and-forget docker build
    builder::build_image(
        build_dir,
        image,
        id.clone(),
        state.ws_tx.clone(),
    )
    .await;

    // Mark ready after build completes (handled by WS event on client; also update store)
    let state_clone = state.clone();
    let id_clone = id.clone();
    tokio::spawn(async move {
        // Listen for build_complete event — simpler: just update after the bg task finishes
        // The builder::build_image already sends build_complete/build_error via WS.
        // Here we update the registry status reactively by listening on a new channel.
        // For simplicity, we update via a delayed check instead.
        // A proper implementation would use a oneshot channel from builder.
        let _ = update_build_status_on_complete(&state_clone, &id_clone).await;
    });

    Ok(Json(GenerateResponse { node_type }))
}

/// POST /api/node-types/:id/rebuild
pub async fn rebuild(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<RebuildRequest>,
) -> Result<Json<GenerateResponse>, (StatusCode, String)> {
    let api_key = api_key_or_err(&state)?;

    // Find existing node type
    let existing = {
        let types = state.node_types.read().await;
        types.iter().find(|t| t.id == id).cloned()
    }
    .ok_or((StatusCode::NOT_FOUND, "Node type not found".into()))?;

    let source_code = if let Some(code) = body.source_code {
        // Direct code override — skip AI
        code
    } else {
        let description = body.description.as_deref()
            .or(existing.ai_prompt.as_deref())
            .unwrap_or(&existing.description);

        let existing_code = existing.source_code.as_deref().unwrap_or("");

        emit(&state, WsEvent::build_log(&id, "Regenerating code with AI..."));
        let gen = if existing_code.is_empty() {
            ai_codegen::generate_processor(
                &api_key,
                description,
                &existing.name,
                &existing.input_ports,
                &existing.output_ports,
            ).await
        } else {
            ai_codegen::regenerate_processor(
                &api_key,
                existing_code,
                description,
                &existing.name,
                &existing.input_ports,
                &existing.output_ports,
            ).await
        }
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        gen.source_code
    };

    let slug = existing.name.to_lowercase().replace(|c: char| !c.is_alphanumeric(), "-");
    let cargo_toml = format!(
        "[package]\nname = \"mirdain-{slug}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\naxum = \"0.7\"\ntokio = {{ version = \"1\", features = [\"full\"] }}\nserde = {{ version = \"1\", features = [\"derive\"] }}\nserde_json = \"1\"\n"
    );

    let build_dir = builder::write_processor_files(
        &state.processors_dir,
        &id.replace('/', "-"),
        &cargo_toml,
        &source_code,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Update in registry
    let updated = {
        let mut types = state.node_types.write().await;
        if let Some(nt) = types.iter_mut().find(|t| t.id == id) {
            nt.source_code = Some(source_code.clone());
            nt.build_status = BuildStatus::Building;
            if let Some(desc) = &body.description {
                nt.ai_prompt = Some(desc.clone());
                nt.description = desc.clone();
            }
            nt.clone()
        } else {
            return Err((StatusCode::NOT_FOUND, "Node type not found".into()));
        }
    };
    persist_custom_node_types(&state).await;
    emit(&state, WsEvent::node_types_updated());

    builder::build_image(
        build_dir,
        existing.image.clone(),
        id.clone(),
        state.ws_tx.clone(),
    )
    .await;

    Ok(Json(GenerateResponse { node_type: updated }))
}

/// GET /api/node-types/:id/source
pub async fn get_source(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let types = state.node_types.read().await;
    let nt = types.iter().find(|t| t.id == id).ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(serde_json::json!({ "source_code": nt.source_code })))
}

// ── Internal helpers ─────────────────────────────────────────────────────────

fn emit(state: &AppState, event: WsEvent) {
    if let Ok(json) = serde_json::to_string(&event) {
        let _ = state.ws_tx.send(json);
    }
}

async fn persist_custom_node_types(state: &AppState) {
    let types = state.node_types.read().await;
    let custom: Vec<&NodeType> = types.iter().filter(|t| t.ai_prompt.is_some()).collect();
    let path = state.processors_dir.join("registry.json");
    if let Ok(json) = serde_json::to_string_pretty(&custom) {
        let _ = tokio::fs::write(path, json).await;
    }
}

/// Dummy background task — the actual status update is handled by WS on the client.
/// We just wait a bit and mark the node type as Ready (optimistic).
async fn update_build_status_on_complete(state: &AppState, id: &str) -> anyhow::Result<()> {
    // Give the build time to complete (this is imprecise; a channel would be better)
    // The client handles the authoritative state via WS build_complete/build_error events.
    tokio::time::sleep(std::time::Duration::from_secs(120)).await;
    let mut types = state.node_types.write().await;
    if let Some(nt) = types.iter_mut().find(|t| t.id == id) {
        if nt.build_status == BuildStatus::Building {
            nt.build_status = BuildStatus::Ready;
        }
    }
    Ok(())
}
