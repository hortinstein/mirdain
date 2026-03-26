use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::models::Port;

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const MODEL: &str = "claude-haiku-4-5-20251001";

#[derive(Serialize)]
struct AnthropicRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    system: &'a str,
    messages: Vec<Message<'a>>,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    kind: String,
    text: Option<String>,
}

pub struct Generated {
    pub source_code: String,
    pub cargo_toml: String,
}

const SYSTEM: &str = r#"
You are a Rust code generator for the mirdain data-processing framework.
You produce ONLY raw Rust source code with no markdown, no code fences, no explanations.
The generated code must compile with Rust 1.77+ and use axum 0.7 + tokio + serde_json.
"#;

fn cargo_toml(slug: &str) -> String {
    format!(
        r#"[package]
name = "mirdain-{slug}"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = {{ version = "1", features = ["full"] }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
"#
    )
}

/// Generate Rust processor source code from a plain-English description.
pub async fn generate_processor(
    api_key: &str,
    description: &str,
    name: &str,
    input_ports: &[Port],
    output_ports: &[Port],
) -> Result<Generated> {
    let inputs_desc = input_ports
        .iter()
        .map(|p| format!(r#"  - id: "{}", label: "{}""#, p.id, p.label))
        .collect::<Vec<_>>()
        .join("\n");
    let outputs_desc = output_ports
        .iter()
        .map(|p| format!(r#"  - id: "{}", label: "{}""#, p.id, p.label))
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        r#"Generate a complete Rust source file (src/main.rs) for a mirdain data processor.

PROCESSOR NAME: {name}
PROCESSOR DESCRIPTION: {description}

INPUT PORTS:
{inputs_desc}

OUTPUT PORTS:
{outputs_desc}

PROTOCOL REQUIREMENTS:
1. Listen on port from PORT env var, default 3000.
2. Implement GET /health  → {{ "status": "ok" }}
3. Implement GET /manifest → {{
     "name": "{name}",
     "description": "...",
     "version": "1.0.0",
     "inputs":  [{{ "id": "<id>", "label": "<label>" }}, ...],
     "outputs": [{{ "id": "<id>", "label": "<label>" }}, ...]
   }}
4. Implement POST /process that accepts:
   {{ "inputs": {{ "<port_id>": <value>, ... }}, "config": {{ "<key>": <value>, ... }} }}
   and returns:
   {{ "outputs": {{ "<port_id>": <value>, ... }} }}
5. Data values are arbitrary serde_json::Value.
6. Use idiomatic Rust; derive Serialize/Deserialize for request/response structs.
7. Any config fields should also be readable from env vars as a fallback.

Output ONLY the raw Rust source code for src/main.rs. No markdown fences."#
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;

    let body = AnthropicRequest {
        model: MODEL,
        max_tokens: 4096,
        system: SYSTEM,
        messages: vec![Message {
            role: "user",
            content: &prompt,
        }],
    };

    let resp = client
        .post(ANTHROPIC_API_URL)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .send()
        .await
        .map_err(|e| anyhow!("Anthropic request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(anyhow!("Anthropic API {status}: {text}"));
    }

    let api_resp: AnthropicResponse = resp
        .json()
        .await
        .map_err(|e| anyhow!("Anthropic parse error: {e}"))?;

    let source_code = api_resp
        .content
        .into_iter()
        .filter(|c| c.kind == "text")
        .filter_map(|c| c.text)
        .collect::<Vec<_>>()
        .join("");

    // Strip accidental markdown fences
    let source_code = strip_fences(&source_code);

    let slug = name.to_lowercase().replace(|c: char| !c.is_alphanumeric(), "-");

    Ok(Generated {
        source_code,
        cargo_toml: cargo_toml(&slug),
    })
}

/// Regenerate code with a new description (pass existing code for context).
pub async fn regenerate_processor(
    api_key: &str,
    existing_code: &str,
    new_description: &str,
    name: &str,
    input_ports: &[Port],
    output_ports: &[Port],
) -> Result<Generated> {
    let inputs_desc = input_ports
        .iter()
        .map(|p| format!(r#"  - id: "{}", label: "{}""#, p.id, p.label))
        .collect::<Vec<_>>()
        .join("\n");
    let outputs_desc = output_ports
        .iter()
        .map(|p| format!(r#"  - id: "{}", label: "{}""#, p.id, p.label))
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        r#"Rewrite the following mirdain Rust processor to match the new description.
Preserve the axum 0.7 + tokio + serde_json stack and the /health, /manifest, /process endpoints.

PROCESSOR NAME: {name}
NEW DESCRIPTION: {new_description}

INPUT PORTS:
{inputs_desc}

OUTPUT PORTS:
{outputs_desc}

EXISTING CODE:
```rust
{existing_code}
```

Output ONLY the raw updated Rust source code. No markdown fences."#
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;

    let body = AnthropicRequest {
        model: MODEL,
        max_tokens: 4096,
        system: SYSTEM,
        messages: vec![Message {
            role: "user",
            content: &prompt,
        }],
    };

    let resp = client
        .post(ANTHROPIC_API_URL)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .send()
        .await
        .map_err(|e| anyhow!("Anthropic request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(anyhow!("Anthropic API {status}: {text}"));
    }

    let api_resp: AnthropicResponse = resp
        .json()
        .await
        .map_err(|e| anyhow!("Anthropic parse error: {e}"))?;

    let source_code = api_resp
        .content
        .into_iter()
        .filter(|c| c.kind == "text")
        .filter_map(|c| c.text)
        .collect::<Vec<_>>()
        .join("");

    let source_code = strip_fences(&source_code);

    let slug = name.to_lowercase().replace(|c: char| !c.is_alphanumeric(), "-");

    Ok(Generated {
        source_code,
        cargo_toml: cargo_toml(&slug),
    })
}

fn strip_fences(s: &str) -> String {
    let s = s.trim();
    // Remove ```rust ... ``` or ``` ... ```
    if let Some(inner) = s
        .strip_prefix("```rust\n")
        .or_else(|| s.strip_prefix("```\n"))
        .or_else(|| s.strip_prefix("```rust"))
        .or_else(|| s.strip_prefix("```"))
    {
        if let Some(stripped) = inner.strip_suffix("\n```").or_else(|| inner.strip_suffix("```")) {
            return stripped.to_string();
        }
    }
    s.to_string()
}
