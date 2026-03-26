use anyhow::{anyhow, Result};
use bollard::container::{
    Config, CreateContainerOptions, RemoveContainerOptions, StartContainerOptions,
    StopContainerOptions,
};
use bollard::models::{HostConfig, PortBinding};
use bollard::network::{CreateNetworkOptions, InspectNetworkOptions};
use bollard::Docker;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;

use crate::models::FlowNode;

const CONTAINER_PORT: u16 = 3000;
const HOST_PORT_START: u16 = 12000;

pub struct DockerManager {
    docker: Docker,
    /// node_id -> host_port
    port_map: Arc<DashMap<String, u16>>,
    next_port: Arc<AtomicU16>,
}

impl DockerManager {
    pub fn new() -> Result<Self> {
        let docker = Docker::connect_with_local_defaults()
            .map_err(|e| anyhow!("Cannot connect to Docker daemon: {e}"))?;
        Ok(Self {
            docker,
            port_map: Arc::new(DashMap::new()),
            next_port: Arc::new(AtomicU16::new(HOST_PORT_START)),
        })
    }

    pub fn host_port_for(&self, node_id: &str) -> Option<u16> {
        self.port_map.get(node_id).map(|p| *p)
    }

    fn alloc_port(&self) -> u16 {
        self.next_port.fetch_add(1, Ordering::SeqCst)
    }

    fn container_name(pipeline_id: &str, node_id: &str) -> String {
        let pid = &pipeline_id[..pipeline_id.len().min(8)];
        let nid = &node_id[..node_id.len().min(8)];
        format!("mirdain-{pid}-{nid}")
    }

    fn network_name(pipeline_id: &str) -> String {
        let pid = &pipeline_id[..pipeline_id.len().min(8)];
        format!("mirdain-net-{pid}")
    }

    pub async fn ensure_network(&self, pipeline_id: &str) -> Result<String> {
        let name = Self::network_name(pipeline_id);
        let existing = self
            .docker
            .inspect_network(&name, None::<InspectNetworkOptions<String>>)
            .await;
        if existing.is_err() {
            self.docker
                .create_network(CreateNetworkOptions {
                    name: name.clone(),
                    ..Default::default()
                })
                .await?;
        }
        Ok(name)
    }

    pub async fn remove_network(&self, pipeline_id: &str) -> Result<()> {
        let name = Self::network_name(pipeline_id);
        let _ = self.docker.remove_network(&name).await;
        Ok(())
    }

    pub async fn start_node(
        &self,
        pipeline_id: &str,
        node: &FlowNode,
        network: &str,
    ) -> Result<u16> {
        let container_name = Self::container_name(pipeline_id, &node.id);
        let host_port = self.alloc_port();

        // Build env from config
        let env: Vec<String> = node
            .config
            .iter()
            .map(|(k, v)| {
                let val = v
                    .as_str()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| v.to_string());
                format!("{}={}", k.to_uppercase(), val)
            })
            .chain(std::iter::once(format!("PORT={CONTAINER_PORT}")))
            .collect();

        let mut port_bindings = HashMap::new();
        port_bindings.insert(
            format!("{CONTAINER_PORT}/tcp"),
            Some(vec![PortBinding {
                host_ip: Some("127.0.0.1".into()),
                host_port: Some(host_port.to_string()),
            }]),
        );

        let config = Config {
            image: Some(node.image.clone()),
            hostname: Some(format!("node-{}", &node.id[..node.id.len().min(8)])),
            env: Some(env),
            host_config: Some(HostConfig {
                network_mode: Some(network.to_string()),
                port_bindings: Some(port_bindings),
                auto_remove: Some(true),
                ..Default::default()
            }),
            exposed_ports: Some({
                let mut m = HashMap::new();
                m.insert(format!("{CONTAINER_PORT}/tcp"), HashMap::new());
                m
            }),
            ..Default::default()
        };

        // Remove stale container if it exists
        let _ = self
            .docker
            .remove_container(
                &container_name,
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await;

        self.docker
            .create_container(
                Some(CreateContainerOptions {
                    name: container_name.clone(),
                    ..Default::default()
                }),
                config,
            )
            .await?;

        self.docker
            .start_container(&container_name, None::<StartContainerOptions<String>>)
            .await?;

        self.port_map.insert(node.id.clone(), host_port);
        Ok(host_port)
    }

    pub async fn stop_node(&self, pipeline_id: &str, node_id: &str) -> Result<()> {
        let container_name = Self::container_name(pipeline_id, node_id);
        let _ = self
            .docker
            .stop_container(
                &container_name,
                Some(StopContainerOptions { t: 5 }),
            )
            .await;
        self.port_map.remove(node_id);
        Ok(())
    }

    pub async fn stop_all_nodes(&self, pipeline_id: &str, node_ids: &[String]) -> Result<()> {
        for nid in node_ids {
            self.stop_node(pipeline_id, nid).await?;
        }
        Ok(())
    }
}
