use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use tokio::io::AsyncBufReadExt;
use tokio::process::Command;
use tokio::sync::broadcast;

use crate::models::WsEvent;

const DOCKERFILE: &str = r#"FROM rust:1.77-slim as builder
WORKDIR /app
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/* /usr/local/bin/processor
ENV PORT=3000
EXPOSE 3000
CMD ["processor"]
"#;

/// Write source files to disk for a processor.
pub async fn write_processor_files(
    processors_dir: &Path,
    node_id: &str,
    cargo_toml: &str,
    source_code: &str,
) -> Result<PathBuf> {
    let dir = processors_dir.join(node_id);
    let src_dir = dir.join("src");
    tokio::fs::create_dir_all(&src_dir).await?;

    tokio::fs::write(dir.join("Cargo.toml"), cargo_toml).await?;
    tokio::fs::write(src_dir.join("main.rs"), source_code).await?;
    tokio::fs::write(dir.join("Dockerfile"), DOCKERFILE).await?;
    tokio::fs::write(dir.join("source_code.rs"), source_code).await?;

    Ok(dir)
}

/// Read previously generated source code for a node type.
pub async fn read_source_code(processors_dir: &Path, node_id: &str) -> Option<String> {
    let path = processors_dir.join(node_id).join("source_code.rs");
    tokio::fs::read_to_string(path).await.ok()
}

/// Build a Docker image from a processor directory, streaming logs via WebSocket.
pub async fn build_image(
    build_dir: PathBuf,
    image_tag: String,
    node_type_id: String,
    ws_tx: broadcast::Sender<String>,
) {
    tokio::spawn(async move {
        emit(&ws_tx, WsEvent::build_log(&node_type_id, &format!("Building {image_tag} ...")));

        let result = run_docker_build(&build_dir, &image_tag, &node_type_id, &ws_tx).await;

        match result {
            Ok(_) => {
                emit(
                    &ws_tx,
                    WsEvent::build_complete(&node_type_id, &image_tag),
                );
            }
            Err(e) => {
                emit(&ws_tx, WsEvent::build_error(&node_type_id, &e.to_string()));
            }
        }
    });
}

async fn run_docker_build(
    build_dir: &Path,
    image_tag: &str,
    node_type_id: &str,
    ws_tx: &broadcast::Sender<String>,
) -> Result<()> {
    let mut child = Command::new("docker")
        .args(["build", "--no-cache", "-t", image_tag, "."])
        .current_dir(build_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| anyhow!("Failed to spawn docker: {e}"))?;

    // Stream stdout
    if let Some(stdout) = child.stdout.take() {
        let tx = ws_tx.clone();
        let id = node_type_id.to_string();
        tokio::spawn(async move {
            let mut lines = tokio::io::BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                emit(&tx, WsEvent::build_log(&id, &line));
            }
        });
    }

    // Stream stderr
    if let Some(stderr) = child.stderr.take() {
        let tx = ws_tx.clone();
        let id = node_type_id.to_string();
        tokio::spawn(async move {
            let mut lines = tokio::io::BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                emit(&tx, WsEvent::build_log(&id, &line));
            }
        });
    }

    let status = child.wait().await?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "docker build exited with code {}",
            status.code().unwrap_or(-1)
        ))
    }
}

fn emit(tx: &broadcast::Sender<String>, event: WsEvent) {
    if let Ok(json) = serde_json::to_string(&event) {
        let _ = tx.send(json);
    }
}
