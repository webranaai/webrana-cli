// ============================================
// WEBRANA CLI - Context/Memory Management
// Sprint 5.1: Optimized context window
// Created by: FORGE (Team Beta)
// ============================================

use crate::llm::Message;

/// Configuration for context window management
#[derive(Debug, Clone)]
pub struct ContextConfig {
    /// Maximum number of messages to keep
    pub max_messages: usize,
    /// Maximum total characters (approximate token limit)
    pub max_chars: usize,
    /// Keep at least this many recent messages
    pub min_recent_messages: usize,
    /// Summarize old context when trimming
    pub enable_summarization: bool,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_messages: 50,
            max_chars: 100_000, // ~25k tokens
            min_recent_messages: 5,
            enable_summarization: false,
        }
    }
}

/// Optimized context window management
#[derive(Debug, Clone)]
pub struct Context {
    messages: Vec<Message>,
    config: ContextConfig,
    total_chars: usize,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            config: ContextConfig::default(),
            total_chars: 0,
        }
    }

    pub fn with_config(config: ContextConfig) -> Self {
        Self {
            messages: Vec::new(),
            config,
            total_chars: 0,
        }
    }

    pub fn with_max_messages(max: usize) -> Self {
        Self {
            messages: Vec::new(),
            config: ContextConfig {
                max_messages: max,
                ..Default::default()
            },
            total_chars: 0,
        }
    }

    pub fn add_user_message(&mut self, content: &str) {
        self.add_message(Message::user(content));
    }

    pub fn add_assistant_message(&mut self, content: &str) {
        self.add_message(Message::assistant(content));
    }

    pub fn add_system_message(&mut self, content: &str) {
        self.add_message(Message::system(content));
    }

    fn add_message(&mut self, message: Message) {
        self.total_chars += message.content.len();
        self.messages.push(message);
        self.optimize();
    }

    /// Smart context optimization
    fn optimize(&mut self) {
        // First, trim by message count
        while self.messages.len() > self.config.max_messages {
            if let Some(removed) = self.messages.first() {
                self.total_chars = self.total_chars.saturating_sub(removed.content.len());
            }
            self.messages.remove(0);
        }

        // Then, trim by character count while keeping minimum recent messages
        while self.total_chars > self.config.max_chars 
            && self.messages.len() > self.config.min_recent_messages 
        {
            if let Some(removed) = self.messages.first() {
                self.total_chars = self.total_chars.saturating_sub(removed.content.len());
            }
            self.messages.remove(0);
        }
    }

    /// Get messages optimized for token budget
    pub fn get_messages(&self) -> &[Message] {
        &self.messages
    }

    /// Get messages with a specific token budget (chars * 0.25 ≈ tokens)
    pub fn get_messages_for_budget(&self, max_chars: usize) -> Vec<Message> {
        let mut result = Vec::new();
        let mut chars = 0;

        // Add messages from most recent, respecting budget
        for msg in self.messages.iter().rev() {
            if chars + msg.content.len() <= max_chars {
                chars += msg.content.len();
                result.push(msg.clone());
            } else if result.is_empty() {
                // Always include at least the most recent message (truncated if needed)
                let mut truncated = msg.clone();
                if truncated.content.len() > max_chars {
                    truncated.content = truncated.content[..max_chars].to_string();
                }
                result.push(truncated);
                break;
            } else {
                break;
            }
        }

        result.reverse();
        result
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.total_chars = 0;
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Get approximate token count (chars / 4)
    pub fn estimated_tokens(&self) -> usize {
        self.total_chars / 4
    }

    /// Get total character count
    pub fn total_chars(&self) -> usize {
        self.total_chars
    }

    /// Get context statistics
    pub fn stats(&self) -> ContextStats {
        ContextStats {
            message_count: self.messages.len(),
            total_chars: self.total_chars,
            estimated_tokens: self.estimated_tokens(),
            max_messages: self.config.max_messages,
            max_chars: self.config.max_chars,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContextStats {
    pub message_count: usize,
    pub total_chars: usize,
    pub estimated_tokens: usize,
    pub max_messages: usize,
    pub max_chars: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_basic() {
        let mut ctx = Context::new();
        ctx.add_user_message("Hello");
        ctx.add_assistant_message("Hi there!");
        
        assert_eq!(ctx.len(), 2);
        assert!(!ctx.is_empty());
    }

    #[test]
    fn test_context_trim_by_count() {
        let mut ctx = Context::with_max_messages(3);
        ctx.add_user_message("1");
        ctx.add_user_message("2");
        ctx.add_user_message("3");
        ctx.add_user_message("4");
        
        assert_eq!(ctx.len(), 3);
        assert_eq!(ctx.get_messages()[0].content, "2");
    }

    #[test]
    fn test_context_trim_by_chars() {
        let config = ContextConfig {
            max_messages: 100,
            max_chars: 20,
            min_recent_messages: 1,
            ..Default::default()
        };
        let mut ctx = Context::with_config(config);
        
        ctx.add_user_message("Hello World!"); // 12 chars
        ctx.add_user_message("Another msg"); // 11 chars, total 23 > 20
        
        // Should trim to fit within max_chars
        assert!(ctx.total_chars() <= 20 || ctx.len() <= 1);
    }

    #[test]
    fn test_context_budget() {
        let mut ctx = Context::new();
        ctx.add_user_message("Short");
        ctx.add_user_message("This is a much longer message");
        ctx.add_user_message("Final");
        
        let budget_msgs = ctx.get_messages_for_budget(15);
        assert!(!budget_msgs.is_empty());
    }

    #[test]
    fn test_context_stats() {
        let mut ctx = Context::new();
        ctx.add_user_message("Hello");
        
        let stats = ctx.stats();
        assert_eq!(stats.message_count, 1);
        assert_eq!(stats.total_chars, 5);
    }
}
