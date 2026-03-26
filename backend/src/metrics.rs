use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeMetrics {
    pub messages_in: u64,
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub total_latency_ms: u64,
    pub errors: u64,
}

impl NodeMetrics {
    pub fn avg_latency_ms(&self) -> f64 {
        if self.messages_in == 0 {
            0.0
        } else {
            self.total_latency_ms as f64 / self.messages_in as f64
        }
    }
}

pub type MetricsStore = Arc<DashMap<String, NodeMetrics>>;

pub fn new_store() -> MetricsStore {
    Arc::new(DashMap::new())
}

/// Record one processed message for a node.
pub fn record(
    store: &MetricsStore,
    node_id: &str,
    bytes_in: usize,
    bytes_out: usize,
    latency_ms: u64,
    is_error: bool,
) {
    let mut entry = store.entry(node_id.to_string()).or_default();
    entry.messages_in += 1;
    entry.bytes_in += bytes_in as u64;
    entry.bytes_out += bytes_out as u64;
    entry.total_latency_ms += latency_ms;
    if is_error {
        entry.errors += 1;
    }
}

/// Snapshot of all metrics.
pub fn snapshot(store: &MetricsStore) -> Vec<(String, NodeMetrics)> {
    store
        .iter()
        .map(|r| (r.key().clone(), r.value().clone()))
        .collect()
}

/// Simple timer helper.
pub struct Timer(Instant);

impl Timer {
    pub fn start() -> Self {
        Self(Instant::now())
    }
    pub fn elapsed_ms(&self) -> u64 {
        self.0.elapsed().as_millis() as u64
    }
}
