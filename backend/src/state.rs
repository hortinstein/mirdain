use std::sync::Arc;
use tokio::sync::broadcast;

use crate::docker_mgr::DockerManager;
use crate::models::NodeType;
use crate::store::PipelineStore;

pub struct AppStateInner {
    pub store: PipelineStore,
    pub docker: DockerManager,
    pub ws_tx: broadcast::Sender<String>,
    pub node_types: Vec<NodeType>,
}

pub type AppState = Arc<AppStateInner>;
