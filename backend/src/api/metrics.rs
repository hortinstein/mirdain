use axum::{extract::State, Json};

use crate::metrics::{snapshot, NodeMetrics};
use crate::state::AppState;

pub async fn all(State(state): State<AppState>) -> Json<serde_json::Value> {
    let entries: std::collections::HashMap<String, NodeMetrics> = snapshot(&state.metrics)
        .into_iter()
        .map(|(id, m)| {
            let avg = m.avg_latency_ms();
            // Enrich with derived field inline
            let mut v = serde_json::to_value(&m).unwrap_or_default();
            if let Some(obj) = v.as_object_mut() {
                obj.insert(
                    "avg_latency_ms".to_string(),
                    serde_json::json!(avg),
                );
            }
            (id, serde_json::from_value(v).unwrap_or_default())
        })
        .collect();
    Json(serde_json::json!(entries))
}
