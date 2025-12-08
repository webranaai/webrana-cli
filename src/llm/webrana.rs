use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::path::PathBuf;
use futures_util::StreamExt;

use super::providers::{ChatResponse, Message, Provider, Role, ToolCall, ToolDefinition};

const API_BASE_URL: &str = "https://api.webrana.id";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub token: String,
    pub device_id: String,
    pub tier: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub token: String,
    pub tier: String,
    pub limits: TierLimits,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TierLimits {
    pub requests_per_day: i32,
    pub tokens_per_day: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    pub tier: String,
    pub usage: UsageInfo,
    pub resets_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageInfo {
    pub requests_today: i32,
    pub tokens_today: i32,
    pub requests_limit: i32,
    pub tokens_limit: i32,
}

pub struct WebranaProvider {
    credentials: Credentials,
}

impl WebranaProvider {
    pub async fn new() -> Result<Self> {
        let credentials = Self::load_or_register().await?;
        Ok(Self { credentials })
    }

    fn credentials_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("webrana");
        std::fs::create_dir_all(&config_dir).ok();
        config_dir.join("credentials.json")
    }

    fn load_credentials() -> Option<Credentials> {
        let path = Self::credentials_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path).ok()?;
            serde_json::from_str(&content).ok()
        } else {
            None
        }
    }

    fn save_credentials(creds: &Credentials) -> Result<()> {
        let path = Self::credentials_path();
        let content = serde_json::to_string_pretty(creds)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    fn generate_device_id() -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        
        if let Ok(hostname) = hostname::get() {
            hostname.to_string_lossy().hash(&mut hasher);
        }
        
        if let Ok(username) = std::env::var("USER").or_else(|_| std::env::var("USERNAME")) {
            username.hash(&mut hasher);
        }

        std::env::consts::OS.hash(&mut hasher);
        std::env::consts::ARCH.hash(&mut hasher);

        format!("webrana-{:x}", hasher.finish())
    }

    async fn register() -> Result<Credentials> {
        let client = reqwest::Client::new();
        let device_id = Self::generate_device_id();

        let response = client
            .post(format!("{}/v1/auth/register", API_BASE_URL))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "device_id": device_id,
                "device_name": hostname::get()
                    .map(|h| h.to_string_lossy().to_string())
                    .unwrap_or_else(|_| "Unknown".to_string()),
                "os": std::env::consts::OS,
                "cli_version": env!("CARGO_PKG_VERSION")
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await?;
            return Err(anyhow!("Registration failed: {}", error));
        }

        let reg: RegisterResponse = response.json().await?;
        
        let credentials = Credentials {
            token: reg.token,
            device_id,
            tier: reg.tier,
        };

        Self::save_credentials(&credentials)?;
        
        eprintln!("✓ Registered with Webrana API (tier: {})", credentials.tier);
        
        Ok(credentials)
    }

    async fn load_or_register() -> Result<Credentials> {
        if let Some(creds) = Self::load_credentials() {
            Ok(creds)
        } else {
            Self::register().await
        }
    }

    pub async fn get_status() -> Result<StatusResponse> {
        let credentials = Self::load_or_register().await?;
        let client = reqwest::Client::new();

        let response = client
            .get(format!("{}/v1/auth/status", API_BASE_URL))
            .header("Authorization", format!("Bearer {}", credentials.token))
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await?;
            return Err(anyhow!("Failed to get status: {}", error));
        }

        Ok(response.json().await?)
    }

    pub fn get_credentials() -> Option<Credentials> {
        Self::load_credentials()
    }

    pub fn clear_credentials() -> Result<()> {
        let path = Self::credentials_path();
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }
}

#[async_trait]
impl Provider for WebranaProvider {
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

        let response = client
            .post(format!("{}/v1/chat/completions", API_BASE_URL))
            .header("Authorization", format!("Bearer {}", self.credentials.token))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "messages": chat_messages,
                "stream": false
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await?;
            return Err(anyhow!("Chat request failed: {}", error));
        }

        let json: serde_json::Value = response.json().await?;

        let content = json["choices"][0]["message"]["content"]
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

        let response = client
            .post(format!("{}/v1/chat/completions", API_BASE_URL))
            .header("Authorization", format!("Bearer {}", self.credentials.token))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "messages": chat_messages,
                "stream": true
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await?;
            return Err(anyhow!("Chat request failed: {}", error));
        }

        let mut stream = response.bytes_stream();
        let mut content = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let text = String::from_utf8_lossy(&chunk);

            for line in text.lines() {
                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" {
                        continue;
                    }

                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                        if let Some(delta_content) = json["choices"][0]["delta"]["content"].as_str() {
                            print!("{}", delta_content);
                            io::stdout().flush().ok();
                            content.push_str(delta_content);
                        }
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
        "webrana"
    }
}
