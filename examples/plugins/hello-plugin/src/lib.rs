// Hello Plugin - Example Webrana Plugin
// Demonstrates the plugin system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize)]
struct PluginInput {
    action: String,
    params: serde_json::Value,
}

#[derive(Serialize)]
struct PluginOutput {
    success: bool,
    result: serde_json::Value,
    logs: Vec<String>,
}

#[no_mangle]
pub extern "C" fn execute(input_ptr: *const u8, input_len: usize) -> *mut u8 {
    let input_bytes = unsafe { std::slice::from_raw_parts(input_ptr, input_len) };
    let input_str = std::str::from_utf8(input_bytes).unwrap_or("{}");
    
    let output = match serde_json::from_str::<PluginInput>(input_str) {
        Ok(input) => process_action(&input),
        Err(e) => PluginOutput {
            success: false,
            result: serde_json::json!({ "error": e.to_string() }),
            logs: vec![],
        },
    };

    let output_json = serde_json::to_string(&output).unwrap_or_default();
    let bytes = output_json.into_bytes();
    let ptr = bytes.as_ptr() as *mut u8;
    std::mem::forget(bytes);
    ptr
}

fn process_action(input: &PluginInput) -> PluginOutput {
    match input.action.as_str() {
        "greet" => greet(&input.params),
        "count_files" => count_files(&input.params),
        _ => PluginOutput {
            success: false,
            result: serde_json::json!({ "error": "Unknown action" }),
            logs: vec![],
        },
    }
}

fn greet(params: &serde_json::Value) -> PluginOutput {
    let name = params
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("World");

    let greeting = format!("Hello, {}! Welcome to Webrana CLI.", name);

    PluginOutput {
        success: true,
        result: serde_json::json!({
            "greeting": greeting,
            "name": name
        }),
        logs: vec![format!("Greeted {}", name)],
    }
}

fn count_files(params: &serde_json::Value) -> PluginOutput {
    let path = params
        .get("path")
        .and_then(|v| v.as_str())
        .unwrap_or(".");

    // In WASM, we can't actually access filesystem
    // This is a placeholder that would work with WASI
    let count = 0; // Placeholder
    
    PluginOutput {
        success: true,
        result: serde_json::json!({
            "path": path,
            "count": count,
            "message": format!("Would count files in {} (requires WASI)", path)
        }),
        logs: vec![format!("Counted files in {}", path)],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        let params = serde_json::json!({ "name": "Test" });
        let output = greet(&params);
        assert!(output.success);
        assert!(output.result["greeting"].as_str().unwrap().contains("Test"));
    }

    #[test]
    fn test_greet_default() {
        let params = serde_json::json!({});
        let output = greet(&params);
        assert!(output.success);
        assert!(output.result["greeting"].as_str().unwrap().contains("World"));
    }
}
