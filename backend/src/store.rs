use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;

use crate::models::Pipeline;

pub struct PipelineStore {
    data: RwLock<HashMap<String, Pipeline>>,
    path: PathBuf,
}

impl PipelineStore {
    pub async fn load(path: PathBuf) -> Result<Self> {
        let data: HashMap<String, Pipeline> = if path.exists() {
            let raw = tokio::fs::read_to_string(&path).await?;
            serde_json::from_str(&raw).unwrap_or_default()
        } else {
            HashMap::new()
        };
        Ok(Self {
            data: RwLock::new(data),
            path,
        })
    }

    pub async fn list(&self) -> Vec<Pipeline> {
        self.data.read().await.values().cloned().collect()
    }

    pub async fn get(&self, id: &str) -> Option<Pipeline> {
        self.data.read().await.get(id).cloned()
    }

    pub async fn upsert(&self, pipeline: Pipeline) -> Result<()> {
        self.data
            .write()
            .await
            .insert(pipeline.id.clone(), pipeline);
        self.flush().await
    }

    pub async fn delete(&self, id: &str) -> Result<bool> {
        let removed = self.data.write().await.remove(id).is_some();
        if removed {
            self.flush().await?;
        }
        Ok(removed)
    }

    async fn flush(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let json = serde_json::to_string_pretty(&*self.data.read().await)?;
        tokio::fs::write(&self.path, json).await?;
        Ok(())
    }
}
