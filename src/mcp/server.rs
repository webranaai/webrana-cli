use anyhow::Result;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

use crate::config::Settings;
use crate::skills::SkillRegistry;
use super::protocol::*;

pub async fn start(port: u16) -> Result<()> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    println!("MCP Server listening on port {}", port);

    let settings = Settings::load()?;
    let skills = SkillRegistry::new();

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from: {}", addr);

        let settings = settings.clone();
        let skills_defs = skills.to_tool_definitions();

        tokio::spawn(async move {
            let (reader, mut writer) = socket.into_split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => break, // Connection closed
                    Ok(_) => {
                        if let Ok(request) = serde_json::from_str::<McpRequest>(&line) {
                            let response = handle_request(&request, &settings, &skills_defs).await;
                            let response_json = serde_json::to_string(&response).unwrap();
                            let _ = writer.write_all(response_json.as_bytes()).await;
                            let _ = writer.write_all(b"\n").await;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }
}

async fn handle_request(
    request: &McpRequest,
    _settings: &Settings,
    tools: &[Value],
) -> McpResponse {
    match request.method.as_str() {
        "initialize" => {
            McpResponse::success(
                request.id.clone(),
                json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {},
                        "resources": {},
                        "prompts": {}
                    },
                    "serverInfo": {
                        "name": "webrana",
                        "version": "0.1.0"
                    }
                }),
            )
        }

        "tools/list" => {
            McpResponse::success(
                request.id.clone(),
                json!({
                    "tools": tools
                }),
            )
        }

        "tools/call" => {
            if let Some(params) = &request.params {
                let tool_name = params["name"].as_str().unwrap_or("");
                let tool_args = params.get("arguments").cloned().unwrap_or(json!({}));

                // Execute the tool
                let skills = SkillRegistry::new();
                let settings = Settings::load().unwrap_or_default();
                
                match skills.execute(tool_name, &tool_args, &settings).await {
                    Ok(result) => McpResponse::success(
                        request.id.clone(),
                        json!({
                            "content": [{
                                "type": "text",
                                "text": result
                            }]
                        }),
                    ),
                    Err(e) => McpResponse::error(
                        request.id.clone(),
                        INTERNAL_ERROR,
                        &e.to_string(),
                    ),
                }
            } else {
                McpResponse::error(request.id.clone(), INVALID_PARAMS, "Missing parameters")
            }
        }

        "resources/list" => {
            McpResponse::success(
                request.id.clone(),
                json!({
                    "resources": []
                }),
            )
        }

        "prompts/list" => {
            McpResponse::success(
                request.id.clone(),
                json!({
                    "prompts": []
                }),
            )
        }

        _ => McpResponse::error(
            request.id.clone(),
            METHOD_NOT_FOUND,
            &format!("Method not found: {}", request.method),
        ),
    }
}
