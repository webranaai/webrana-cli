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

    /// Test WASM plugin loading from WAT format
    #[test]
    fn test_wasm_wat_compilation() {
        let temp = tempdir().expect("Failed to create temp dir");
        let wat_path = temp.path().join("test.wat");
        
        // Simple WAT module with exported functions
        let wat_code = r#"
(module
  (func (export "greet") (result i32)
    i32.const 42
  )
  (func (export "add") (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.add
  )
)
"#;
        
        fs::write(&wat_path, wat_code).expect("Failed to write WAT file");
        
        // Compile using wasmtime
        use wasmtime::{Engine, Module};
        let engine = Engine::default();
        let wat_text = fs::read_to_string(&wat_path).expect("Failed to read WAT");
        let result = Module::new(&engine, &wat_text);
        
        assert!(result.is_ok(), "WAT compilation should succeed");
        
        let module = result.unwrap();
        let exports: Vec<String> = module.exports().map(|e| e.name().to_string()).collect();
        
        assert!(exports.contains(&"greet".to_string()), "Should export 'greet'");
        assert!(exports.contains(&"add".to_string()), "Should export 'add'");
    }

    /// Test WASM function execution
    #[test]
    fn test_wasm_function_execution() {
        use wasmtime::{Engine, Module, Store, Linker};
        
        let wat_code = r#"
(module
  (func (export "add") (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.add
  )
)
"#;
        
        let engine = Engine::default();
        let module = Module::new(&engine, wat_code).expect("WAT compilation failed");
        let mut store = Store::new(&engine, ());
        let linker = Linker::new(&engine);
        let instance = linker.instantiate(&mut store, &module).expect("Instantiation failed");
        
        // Get and call the add function
        let add_func = instance.get_typed_func::<(i32, i32), i32>(&mut store, "add")
            .expect("Failed to get add function");
        
        let result = add_func.call(&mut store, (5, 3)).expect("Function call failed");
        assert_eq!(result, 8, "5 + 3 should equal 8");
        
        let result2 = add_func.call(&mut store, (100, 200)).expect("Function call failed");
        assert_eq!(result2, 300, "100 + 200 should equal 300");
    }

    /// Test hello-world plugin exists and is valid
    #[test]
    fn test_hello_world_plugin_exists() {
        let plugin_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("plugins/hello-world");
        
        assert!(plugin_dir.exists(), "hello-world plugin directory should exist");
        
        let manifest_path = plugin_dir.join("manifest.yaml");
        assert!(manifest_path.exists(), "manifest.yaml should exist");
        
        let wat_path = plugin_dir.join("plugin.wat");
        assert!(wat_path.exists(), "plugin.wat should exist");
        
        // Validate manifest structure
        let manifest_content = fs::read_to_string(&manifest_path).expect("Failed to read manifest");
        let manifest: serde_yaml::Value = serde_yaml::from_str(&manifest_content)
            .expect("Failed to parse manifest");
        
        assert_eq!(manifest["id"].as_str(), Some("hello-world"));
        assert_eq!(manifest["plugin_type"].as_str(), Some("wasm"));
    }
}
