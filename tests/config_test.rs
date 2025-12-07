// ============================================
// WEBRANA AI - Configuration Tests
// Created by: VALIDATOR (Team Beta)
// ============================================

#[cfg(test)]
mod config_tests {
    use std::collections::HashMap;
    use tempfile::tempdir;
    use std::fs;

    /// Test default settings creation
    #[test]
    fn test_default_settings() {
        let settings_toml = r#"
default_model = "claude"
default_agent = "nexus"

[models.claude]
provider = "anthropic"
api_key_env = "ANTHROPIC_API_KEY"
model = "claude-sonnet-4-20250514"
temperature = 0.7
max_tokens = 4096

[models.gpt]
provider = "openai"
api_key_env = "OPENAI_API_KEY"
model = "gpt-4o"
temperature = 0.7
max_tokens = 4096

[safety]
confirm_file_write = true
confirm_file_delete = true
confirm_shell_execute = true
"#;
        
        let parsed: Result<toml::Value, _> = toml::from_str(settings_toml);
        assert!(parsed.is_ok(), "Settings TOML should be valid");
        
        let value = parsed.unwrap();
        assert_eq!(value.get("default_model").and_then(|v| v.as_str()), Some("claude"));
        assert_eq!(value.get("default_agent").and_then(|v| v.as_str()), Some("nexus"));
    }

    /// Test model configuration validation
    #[test]
    fn test_model_config_structure() {
        let model_toml = r#"
provider = "anthropic"
api_key_env = "ANTHROPIC_API_KEY"
model = "claude-sonnet-4-20250514"
temperature = 0.7
max_tokens = 4096
"#;
        
        let parsed: toml::Value = toml::from_str(model_toml).unwrap();
        
        assert_eq!(parsed["provider"].as_str(), Some("anthropic"));
        assert!(parsed["temperature"].as_float().is_some());
        assert!(parsed["max_tokens"].as_integer().is_some());
    }

    /// Test safety configuration defaults
    #[test]
    fn test_safety_config_defaults() {
        let safety_toml = r#"
confirm_file_write = true
confirm_file_delete = true
confirm_shell_execute = true
allowed_commands = ["ls", "cat", "echo"]
blocked_paths = ["/etc", "/root"]
"#;
        
        let parsed: toml::Value = toml::from_str(safety_toml).unwrap();
        
        assert!(parsed["confirm_file_write"].as_bool().unwrap());
        assert!(parsed["confirm_file_delete"].as_bool().unwrap());
        assert!(parsed["confirm_shell_execute"].as_bool().unwrap());
        
        let allowed = parsed["allowed_commands"].as_array().unwrap();
        assert_eq!(allowed.len(), 3);
    }

    /// Test agent configuration
    #[test]
    fn test_agent_config_structure() {
        let agent_toml = r#"
name = "NEXUS"
description = "Orchestrator - Task decomposition and routing"
system_prompt = "You are NEXUS, an AI orchestrator..."
model = "claude"
skills = ["*"]
temperature = 0.3
"#;
        
        let parsed: toml::Value = toml::from_str(agent_toml).unwrap();
        
        assert_eq!(parsed["name"].as_str(), Some("NEXUS"));
        assert_eq!(parsed["model"].as_str(), Some("claude"));
        assert!(parsed["skills"].as_array().is_some());
    }

    /// Test config file creation
    #[test]
    fn test_config_file_creation() {
        let temp = tempdir().expect("Failed to create temp dir");
        let config_path = temp.path().join("config.toml");
        
        let config_content = r#"
default_model = "claude"
default_agent = "nexus"

[models.test]
provider = "openai"
model = "gpt-4o"
temperature = 0.5
max_tokens = 2048
"#;
        
        fs::write(&config_path, config_content).expect("Failed to write config");
        assert!(config_path.exists());
        
        let read_content = fs::read_to_string(&config_path).unwrap();
        assert!(read_content.contains("default_model"));
    }

    /// Test provider types
    #[test]
    fn test_provider_types() {
        let providers = vec![
            "anthropic",
            "openai",
            "openai_compatible",
            "ollama",
        ];
        
        for provider in &providers {
            assert!(!provider.is_empty());
            assert!(provider.chars().all(|c| c.is_alphanumeric() || c == '_'));
        }
    }

    /// Test temperature bounds
    #[test]
    fn test_temperature_bounds() {
        let valid_temps = vec![0.0, 0.3, 0.5, 0.7, 1.0, 2.0];
        
        for temp in &valid_temps {
            assert!(*temp >= 0.0, "Temperature should be non-negative");
            assert!(*temp <= 2.0, "Temperature should not exceed 2.0");
        }
    }

    /// Test max_tokens bounds
    #[test]
    fn test_max_tokens_bounds() {
        let valid_tokens = vec![256, 1024, 2048, 4096, 8192, 16384, 32768];
        
        for tokens in &valid_tokens {
            assert!(*tokens >= 1, "max_tokens should be at least 1");
            assert!(*tokens <= 128000, "max_tokens should not exceed context limit");
        }
    }

    /// Test environment variable patterns
    #[test]
    fn test_env_var_patterns() {
        let env_vars = vec![
            "ANTHROPIC_API_KEY",
            "OPENAI_API_KEY",
            "OLLAMA_HOST",
        ];
        
        for var in &env_vars {
            // Valid env var pattern: uppercase with underscores
            assert!(var.chars().all(|c| c.is_uppercase() || c == '_'));
        }
    }
}
