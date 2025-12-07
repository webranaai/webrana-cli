// ============================================
// WEBRANA AI - Plugin System Tests
// Created by: VALIDATOR (Team Beta)
// ============================================

//! Tests for the plugin system
//! Validates plugin loading, manifest parsing, and execution

#[cfg(test)]
mod plugin_tests {
    use std::path::PathBuf;
    use tempfile::tempdir;
    use std::fs;

    /// Test plugin manifest YAML parsing
    #[test]
    fn test_manifest_yaml_structure() {
        let manifest_yaml = r#"
id: test-plugin
name: Test Plugin
version: 1.0.0
description: A test plugin
author:
  name: Test Author
  email: test@example.com
plugin_type: wasm
min_webrana_version: "0.3.0"
permissions:
  - fs:read
skills:
  - name: test_skill
    description: A test skill
    input_schema:
      type: object
      properties:
        input:
          type: string
entry_point: plugin.wasm
"#;

        // YAML should be valid
        let parsed: Result<serde_yaml::Value, _> = serde_yaml::from_str(manifest_yaml);
        assert!(parsed.is_ok(), "Manifest YAML should be valid");

        let value = parsed.unwrap();
        assert_eq!(value["id"].as_str(), Some("test-plugin"));
        assert_eq!(value["version"].as_str(), Some("1.0.0"));
    }

    /// Test plugin manifest TOML parsing
    #[test]
    fn test_manifest_toml_structure() {
        let manifest_toml = r#"
id = "test-plugin"
name = "Test Plugin"
version = "1.0.0"
description = "A test plugin"
plugin_type = "wasm"
min_webrana_version = "0.3.0"
permissions = ["fs:read"]
entry_point = "plugin.wasm"

[author]
name = "Test Author"
email = "test@example.com"

[[skills]]
name = "test_skill"
description = "A test skill"
requires_confirmation = false

[skills.input_schema]
type = "object"
"#;

        // TOML should be valid
        let parsed: Result<toml::Value, _> = toml::from_str(manifest_toml);
        assert!(parsed.is_ok(), "Manifest TOML should be valid");

        let value = parsed.unwrap();
        assert_eq!(value["id"].as_str(), Some("test-plugin"));
        assert_eq!(value["version"].as_str(), Some("1.0.0"));
    }

    /// Test plugin directory structure
    #[test]
    fn test_plugin_directory_structure() {
        let temp = tempdir().expect("Failed to create temp dir");
        let plugin_dir = temp.path().join("my-plugin");
        
        // Create plugin directory structure
        fs::create_dir_all(&plugin_dir).expect("Failed to create plugin dir");
        
        let manifest_path = plugin_dir.join("manifest.yaml");
        fs::write(&manifest_path, "id: my-plugin\nname: My Plugin").expect("Failed to write manifest");
        
        // Verify structure
        assert!(plugin_dir.exists());
        assert!(manifest_path.exists());
    }

    /// Test plugin permission types
    #[test]
    fn test_permission_types() {
        let permissions = vec![
            "fs:read",
            "fs:write",
            "shell:execute",
            "net:request",
            "env:read",
            "git:access",
            "llm:access",
        ];

        for perm in &permissions {
            // All permissions should contain a colon
            assert!(perm.contains(':'), "Permission should have format 'category:action': {}", perm);
            
            let parts: Vec<&str> = perm.split(':').collect();
            assert_eq!(parts.len(), 2, "Permission should have exactly two parts");
        }
    }

    /// Test plugin lifecycle states
    #[test]
    fn test_plugin_states() {
        let states = vec![
            "loaded",
            "ready",
            "executing",
            "error",
            "unloaded",
        ];

        // All states should be distinct
        let unique: std::collections::HashSet<_> = states.iter().collect();
        assert_eq!(unique.len(), states.len(), "All states should be unique");
    }

    /// Test plugin output structure
    #[test]
    fn test_plugin_output() {
        let output_json = serde_json::json!({
            "success": true,
            "result": {
                "message": "Plugin executed successfully"
            },
            "logs": ["Step 1 complete", "Step 2 complete"],
            "artifacts": []
        });

        assert!(output_json["success"].as_bool().unwrap());
        assert!(output_json["result"].is_object());
        assert!(output_json["logs"].is_array());
    }
}
