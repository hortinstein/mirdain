use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use crate::docker_mgr::DockerManager;
use crate::metrics::MetricsStore;
use crate::models::NodeType;
use crate::store::PipelineStore;

pub struct AppStateInner {
    pub store: PipelineStore,
    pub docker: DockerManager,
    pub ws_tx: broadcast::Sender<String>,
    /// Dynamic node type registry (built-ins + AI-generated)
    pub node_types: RwLock<Vec<NodeType>>,
    /// Per-node execution metrics
    pub metrics: MetricsStore,
    /// Anthropic API key (None if not configured)
    pub anthropic_api_key: Option<String>,
    /// Directory where AI-generated processor sources are stored
    pub processors_dir: PathBuf,
}

pub type AppState = Arc<AppStateInner>;
