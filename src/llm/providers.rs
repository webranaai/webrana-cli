use anyhow::Result;
use async_trait::async_trait;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub stop_reason: Option<String>,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
        }
    }
}

#[async_trait]
pub trait Provider: Send + Sync {
    async fn chat(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<ChatResponse>;
    async fn chat_stream(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<ChatResponse>;
    fn name(&self) -> &str;
}

// ============================================================================
// ANTHROPIC PROVIDER (with streaming + tool use)
// ============================================================================

pub struct AnthropicProvider {
    api_key: String,
    model: String,
    max_tokens: u32,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: String, max_tokens: u32) -> Self {
        Self {
            api_key,
            model,
            max_tokens,
        }
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    async fn chat(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<ChatResponse> {
        let client = reqwest::Client::new();

        let system_msg = messages
            .iter()
            .find(|m| m.role == Role::System)
            .map(|m| m.content.clone())
            .unwrap_or_default();

        let chat_messages: Vec<serde_json::Value> = messages
            .iter()
            .filter(|m| m.role != Role::System)
            .map(|m| {
                serde_json::json!({
                    "role": match m.role {
                        Role::User => "user",
                        Role::Assistant => "assistant",
                        Role::System => "user",
                    },
                    "content": m.content
                })
            })
            .collect();

        let mut body = serde_json::json!({
            "model": self.model,
            "max_tokens": self.max_tokens,
            "system": system_msg,
            "messages": chat_messages
        });

        if let Some(tool_defs) = tools {
            let tools_json: Vec<serde_json::Value> = tool_defs
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "name": t.name,
                        "description": t.description,
                        "input_schema": t.input_schema
                    })
                })
                .collect();
            body["tools"] = serde_json::json!(tools_json);
        }

        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;

        let mut content = String::new();
        let mut tool_calls = Vec::new();

        if let Some(contents) = json["content"].as_array() {
            for block in contents {
                match block["type"].as_str() {
                    Some("text") => {
                        if let Some(text) = block["text"].as_str() {
                            content.push_str(text);
                        }
                    }
                    Some("tool_use") => {
                        tool_calls.push(ToolCall {
                            id: block["id"].as_str().unwrap_or("").to_string(),
                            name: block["name"].as_str().unwrap_or("").to_string(),
                            arguments: block["input"].clone(),
                        });
                    }
                    _ => {}
                }
            }
        }

        let stop_reason = json["stop_reason"].as_str().map(String::from);

        Ok(ChatResponse {
            content,
            tool_calls,
            stop_reason,
        })
    }

    async fn chat_stream(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<ChatResponse> {
        let client = reqwest::Client::new();

        let system_msg = messages
            .iter()
            .find(|m| m.role == Role::System)
            .map(|m| m.content.clone())
            .unwrap_or_default();

        let chat_messages: Vec<serde_json::Value> = messages
            .iter()
            .filter(|m| m.role != Role::System)
            .map(|m| {
                serde_json::json!({
                    "role": match m.role {
                        Role::User => "user",
                        Role::Assistant => "assistant",
                        Role::System => "user",
                    },
                    "content": m.content
                })
            })
            .collect();

        let mut body = serde_json::json!({
            "model": self.model,
            "max_tokens": self.max_tokens,
            "system": system_msg,
            "messages": chat_messages,
            "stream": true
        });

        if let Some(tool_defs) = tools {
            let tools_json: Vec<serde_json::Value> = tool_defs
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "name": t.name,
                        "description": t.description,
                        "input_schema": t.input_schema
                    })
                })
                .collect();
            body["tools"] = serde_json::json!(tools_json);
        }

        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        let mut stream = response.bytes_stream();
        let mut content = String::new();
        let mut tool_calls: Vec<ToolCall> = Vec::new();
        let mut current_tool: Option<(String, String, String)> = None; // (id, name, args_json)
        let mut stop_reason = None;
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            // Process complete SSE events
            while let Some(pos) = buffer.find("\n\n") {
                let event = buffer[..pos].to_string();
                buffer = buffer[pos + 2..].to_string();

                for line in event.lines() {
                    if let Some(data) = line.strip_prefix("data: ") {
                        if data == "[DONE]" {
                            continue;
                        }

                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                            match json["type"].as_str() {
                                Some("content_block_start") => {
                                    if json["content_block"]["type"].as_str() == Some("tool_use") {
                                        current_tool = Some((
                                            json["content_block"]["id"]
                                                .as_str()
                                                .unwrap_or("")
                                                .to_string(),
                                            json["content_block"]["name"]
                                                .as_str()
                                                .unwrap_or("")
                                                .to_string(),
                                            String::new(),
                                        ));
                                    }
                                }
                                Some("content_block_delta") => {
                                    if let Some(delta) = json["delta"].as_object() {
                                        if delta.get("type").and_then(|t| t.as_str())
                                            == Some("text_delta")
                                        {
                                            if let Some(text) =
                                                delta.get("text").and_then(|t| t.as_str())
                                            {
                                                print!("{}", text);
                                                io::stdout().flush().ok();
                                                content.push_str(text);
                                            }
                                        } else if delta.get("type").and_then(|t| t.as_str())
                                            == Some("input_json_delta")
                                        {
                                            if let Some((_, _, ref mut args)) = current_tool {
                                                if let Some(partial) = delta
                                                    .get("partial_json")
                                                    .and_then(|p| p.as_str())
                                                {
                                                    args.push_str(partial);
                                                }
                                            }
                                        }
                                    }
                                }
                                Some("content_block_stop") => {
                                    if let Some((id, name, args_str)) = current_tool.take() {
                                        let arguments = serde_json::from_str(&args_str)
                                            .unwrap_or(serde_json::json!({}));
                                        tool_calls.push(ToolCall {
                                            id,
                                            name,
                                            arguments,
                                        });
                                    }
                                }
                                Some("message_delta") => {
                                    if let Some(reason) = json["delta"]["stop_reason"].as_str() {
                                        stop_reason = Some(reason.to_string());
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        println!(); // New line after streaming
        Ok(ChatResponse {
            content,
            tool_calls,
            stop_reason,
        })
    }

    fn name(&self) -> &str {
        "anthropic"
    }
}

// ============================================================================
// OPENAI PROVIDER (with streaming + function calling)
// ============================================================================

pub struct OpenAIProvider {
    api_key: String,
    model: String,
    base_url: String,
}

impl OpenAIProvider {
    pub fn new(api_key: String, model: String, base_url: Option<String>) -> Self {
        Self {
            api_key,
            model,
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
        }
    }
}

#[async_trait]
impl Provider for OpenAIProvider {
    async fn chat(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<ChatResponse> {
        let client = reqwest::Client::new();

        let chat_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": match m.role {
                        Role::System => "system",
                        Role::User => "user",
                        Role::Assistant => "assistant",
                    },
                    "content": m.content
                })
            })
            .collect();

        let mut body = serde_json::json!({
            "model": self.model,
            "messages": chat_messages
        });

        if let Some(tool_defs) = tools {
            let tools_json: Vec<serde_json::Value> = tool_defs
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "type": "function",
                        "function": {
                            "name": t.name,
                            "description": t.description,
                            "parameters": t.input_schema
                        }
                    })
                })
                .collect();
            body["tools"] = serde_json::json!(tools_json);
        }

        let response = client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let mut tool_calls = Vec::new();
        if let Some(calls) = json["choices"][0]["message"]["tool_calls"].as_array() {
            for call in calls {
                tool_calls.push(ToolCall {
                    id: call["id"].as_str().unwrap_or("").to_string(),
                    name: call["function"]["name"].as_str().unwrap_or("").to_string(),
                    arguments: serde_json::from_str(
                        call["function"]["arguments"].as_str().unwrap_or("{}"),
                    )
                    .unwrap_or(serde_json::json!({})),
                });
            }
        }

        let stop_reason = json["choices"][0]["finish_reason"]
            .as_str()
            .map(String::from);

        Ok(ChatResponse {
            content,
            tool_calls,
            stop_reason,
        })
    }

    async fn chat_stream(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<ChatResponse> {
        let client = reqwest::Client::new();

        let chat_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": match m.role {
                        Role::System => "system",
                        Role::User => "user",
                        Role::Assistant => "assistant",
                    },
                    "content": m.content
                })
            })
            .collect();

        let mut body = serde_json::json!({
            "model": self.model,
            "messages": chat_messages,
            "stream": true
        });

        if let Some(tool_defs) = tools {
            let tools_json: Vec<serde_json::Value> = tool_defs
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "type": "function",
                        "function": {
                            "name": t.name,
                            "description": t.description,
                            "parameters": t.input_schema
                        }
                    })
                })
                .collect();
            body["tools"] = serde_json::json!(tools_json);
        }

        let response = client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let mut stream = response.bytes_stream();
        let mut content = String::new();
        let mut tool_calls: Vec<ToolCall> = Vec::new();
        let mut tool_call_map: std::collections::HashMap<usize, (String, String, String)> =
            std::collections::HashMap::new();
        let mut stop_reason = None;
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(pos) = buffer.find("\n") {
                let line = buffer[..pos].to_string();
                buffer = buffer[pos + 1..].to_string();

                if let Some(data) = line.strip_prefix("data: ") {
                    if data.trim() == "[DONE]" {
                        continue;
                    }

                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                        if let Some(delta) = json["choices"][0]["delta"].as_object() {
                            // Text content
                            if let Some(text) = delta.get("content").and_then(|c| c.as_str()) {
                                print!("{}", text);
                                io::stdout().flush().ok();
                                content.push_str(text);
                            }

                            // Tool calls
                            if let Some(calls) = delta.get("tool_calls").and_then(|t| t.as_array())
                            {
                                for call in calls {
                                    let idx = call["index"].as_u64().unwrap_or(0) as usize;

                                    let entry = tool_call_map.entry(idx).or_insert_with(|| {
                                        (
                                            call["id"].as_str().unwrap_or("").to_string(),
                                            String::new(),
                                            String::new(),
                                        )
                                    });

                                    if let Some(name) = call["function"]["name"].as_str() {
                                        entry.1 = name.to_string();
                                    }
                                    if let Some(args) = call["function"]["arguments"].as_str() {
                                        entry.2.push_str(args);
                                    }
                                }
                            }
                        }

                        if let Some(reason) = json["choices"][0]["finish_reason"].as_str() {
                            if !reason.is_empty() && reason != "null" {
                                stop_reason = Some(reason.to_string());
                            }
                        }
                    }
                }
            }
        }

        // Convert tool_call_map to tool_calls vec
        for (_, (id, name, args_str)) in tool_call_map {
            let arguments = serde_json::from_str(&args_str).unwrap_or(serde_json::json!({}));
            tool_calls.push(ToolCall {
                id,
                name,
                arguments,
            });
        }

        println!(); // New line after streaming
        Ok(ChatResponse {
            content,
            tool_calls,
            stop_reason,
        })
    }

    fn name(&self) -> &str {
        "openai"
    }
}

// ============================================================================
// OLLAMA PROVIDER (with streaming)
// ============================================================================

pub struct OllamaProvider {
    base_url: String,
    model: String,
}

impl OllamaProvider {
    pub fn new(base_url: String, model: String) -> Self {
        Self { base_url, model }
    }
}

#[async_trait]
impl Provider for OllamaProvider {
    async fn chat(
        &self,
        messages: Vec<Message>,
        _tools: Option<Vec<ToolDefinition>>,
    ) -> Result<ChatResponse> {
        let client = reqwest::Client::new();

        let chat_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": match m.role {
                        Role::System => "system",
                        Role::User => "user",
                        Role::Assistant => "assistant",
                    },
                    "content": m.content
                })
            })
            .collect();

        let body = serde_json::json!({
            "model": self.model,
            "messages": chat_messages,
            "stream": false
        });

        let response = client
            .post(format!("{}/api/chat", self.base_url))
            .json(&body)
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;

        let content = json["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(ChatResponse {
            content,
            tool_calls: Vec::new(),
            stop_reason: Some("stop".to_string()),
        })
    }

    async fn chat_stream(
        &self,
        messages: Vec<Message>,
        _tools: Option<Vec<ToolDefinition>>,
    ) -> Result<ChatResponse> {
        let client = reqwest::Client::new();

        let chat_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": match m.role {
                        Role::System => "system",
                        Role::User => "user",
                        Role::Assistant => "assistant",
                    },
                    "content": m.content
                })
            })
            .collect();

        let body = serde_json::json!({
            "model": self.model,
            "messages": chat_messages,
            "stream": true
        });

        let response = client
            .post(format!("{}/api/chat", self.base_url))
            .json(&body)
            .send()
            .await?;

        let mut stream = response.bytes_stream();
        let mut content = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let text = String::from_utf8_lossy(&chunk);

            for line in text.lines() {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                    if let Some(msg_content) = json["message"]["content"].as_str() {
                        print!("{}", msg_content);
                        io::stdout().flush().ok();
                        content.push_str(msg_content);
                    }
                }
            }
        }

        println!();
        Ok(ChatResponse {
            content,
            tool_calls: Vec::new(),
            stop_reason: Some("stop".to_string()),
        })
    }

    fn name(&self) -> &str {
        "ollama"
    }
}
