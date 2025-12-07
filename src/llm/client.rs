use anyhow::{Context, Result};
use colored::Colorize;
use std::sync::Arc;

#[allow(unused_imports)]
use super::providers::{
    AnthropicProvider, ChatResponse, Message, OllamaProvider, OpenAIProvider, Provider, ToolCall,
    ToolDefinition,
};
use super::cache::ResponseCache;
use super::retry::{with_retry, RetryConfig};
use crate::config::Settings;
use crate::skills::SkillRegistry;

pub struct LlmClient {
    provider: Arc<dyn Provider>,
    settings: Settings,
    cache: Arc<ResponseCache>,
    retry_config: RetryConfig,
}

impl LlmClient {
    pub fn new(settings: &Settings) -> Result<Self> {
        let model_config = settings
            .get_model(&settings.default_model)
            .context("Default model not found in configuration")?;

        let api_key = settings.get_api_key(model_config);

        let provider: Arc<dyn Provider> = match model_config.provider.as_str() {
            "anthropic" => {
                let key = api_key
                    .context("Anthropic API key not found. Set ANTHROPIC_API_KEY env var.")?;
                Arc::new(AnthropicProvider::new(
                    key,
                    model_config.model.clone(),
                    model_config.max_tokens,
                ))
            }
            "openai" | "openai_compatible" => {
                let key =
                    api_key.context("OpenAI API key not found. Set OPENAI_API_KEY env var.")?;
                Arc::new(OpenAIProvider::new(
                    key,
                    model_config.model.clone(),
                    model_config.base_url.clone(),
                ))
            }
            "ollama" => {
                let base_url = model_config
                    .base_url
                    .clone()
                    .unwrap_or_else(|| "http://localhost:11434".to_string());
                Arc::new(OllamaProvider::new(base_url, model_config.model.clone()))
            }
            _ => anyhow::bail!("Unknown provider: {}", model_config.provider),
        };

        Ok(Self {
            provider,
            settings: settings.clone(),
            cache: Arc::new(ResponseCache::default()),
            retry_config: RetryConfig::default(),
        })
    }

    /// Create client with custom cache and retry configuration
    pub fn with_config(
        settings: &Settings,
        cache: Arc<ResponseCache>,
        retry_config: RetryConfig,
    ) -> Result<Self> {
        let mut client = Self::new(settings)?;
        client.cache = cache;
        client.retry_config = retry_config;
        Ok(client)
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> super::cache::CacheStats {
        self.cache.stats()
    }

    /// Clear the response cache
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    pub async fn chat(
        &self,
        system_prompt: &str,
        history: &[Message],
        user_message: &str,
    ) -> Result<String> {
        let mut messages = vec![Message::system(system_prompt)];
        messages.extend(history.iter().cloned());
        messages.push(Message::user(user_message));

        // Check cache first
        if let Some(cached) = self.cache.get(&messages) {
            tracing::debug!("Cache hit for chat request");
            return Ok(cached);
        }

        // Execute with retry logic
        let provider = self.provider.clone();
        let msgs = messages.clone();
        let response = with_retry(&self.retry_config, || {
            let p = provider.clone();
            let m = msgs.clone();
            async move { p.chat(m, None).await }
        })
        .await?;

        // Cache the response
        self.cache.set(&messages, response.content.clone());

        Ok(response.content)
    }

    pub async fn chat_stream(
        &self,
        system_prompt: &str,
        history: &[Message],
        user_message: &str,
    ) -> Result<String> {
        let mut messages = vec![Message::system(system_prompt)];
        messages.extend(history.iter().cloned());
        messages.push(Message::user(user_message));

        // Streaming responses are not cached (real-time output)
        // But we still use retry logic
        let provider = self.provider.clone();
        let msgs = messages.clone();
        let response = with_retry(&self.retry_config, || {
            let p = provider.clone();
            let m = msgs.clone();
            async move { p.chat_stream(m, None).await }
        })
        .await?;

        Ok(response.content)
    }

    pub async fn chat_with_tools(
        &self,
        system_prompt: &str,
        history: &[Message],
        user_message: &str,
        skill_registry: &SkillRegistry,
    ) -> Result<ChatResponse> {
        let mut messages = vec![Message::system(system_prompt)];
        messages.extend(history.iter().cloned());
        messages.push(Message::user(user_message));

        // Convert skills to tool definitions
        let tools: Vec<ToolDefinition> = skill_registry
            .list()
            .iter()
            .map(|skill| ToolDefinition {
                name: skill.name.clone(),
                description: skill.description.clone(),
                input_schema: skill.parameters.clone(),
            })
            .collect();

        let response = self.provider.chat_stream(messages, Some(tools)).await?;
        Ok(response)
    }

    pub async fn chat_with_tools_loop(
        &self,
        system_prompt: &str,
        history: &mut Vec<Message>,
        user_message: &str,
        skill_registry: &SkillRegistry,
    ) -> Result<String> {
        history.push(Message::user(user_message));

        let mut messages = vec![Message::system(system_prompt)];
        messages.extend(history.iter().cloned());

        // Convert skills to tool definitions
        let tools: Vec<ToolDefinition> = skill_registry
            .list()
            .iter()
            .map(|skill| ToolDefinition {
                name: skill.name.clone(),
                description: skill.description.clone(),
                input_schema: skill.parameters.clone(),
            })
            .collect();

        let max_iterations = 10;
        let mut iteration = 0;
        let mut final_content = String::new();

        loop {
            iteration += 1;
            if iteration > max_iterations {
                println!("\n[Max tool iterations reached]");
                break;
            }

            let response = self
                .provider
                .chat_stream(messages.clone(), Some(tools.clone()))
                .await?;
            final_content = response.content.clone();

            // If no tool calls, we're done
            if response.tool_calls.is_empty() {
                break;
            }

            // Add assistant message with tool calls
            history.push(Message::assistant(&response.content));

            // Execute each tool call
            for tool_call in &response.tool_calls {
                println!(
                    "\n{} Executing tool: {}",
                    "[TOOL]".magenta(),
                    tool_call.name.as_str().cyan()
                );

                let result = skill_registry
                    .execute(&tool_call.name, &tool_call.arguments, &self.settings)
                    .await;

                let result_str = match result {
                    Ok(output) => {
                        println!("{}", output.as_str().dimmed());
                        output
                    }
                    Err(e) => {
                        let err_msg = format!("Error: {}", e);
                        println!("{}", err_msg.as_str().red());
                        err_msg
                    }
                };

                // Add tool result to messages
                // For Anthropic, we need to format this as a user message with tool_result
                let tool_result_msg = format!(
                    "<tool_result tool_use_id=\"{}\">\n{}\n</tool_result>",
                    tool_call.id, result_str
                );
                history.push(Message::user(&tool_result_msg));
            }

            // Update messages for next iteration
            messages = vec![Message::system(system_prompt)];
            messages.extend(history.iter().cloned());
        }

        Ok(final_content)
    }

    pub fn get_tool_definitions(&self, skill_registry: &SkillRegistry) -> Vec<ToolDefinition> {
        skill_registry
            .list()
            .iter()
            .map(|skill| ToolDefinition {
                name: skill.name.clone(),
                description: skill.description.clone(),
                input_schema: skill.parameters.clone(),
            })
            .collect()
    }
}
