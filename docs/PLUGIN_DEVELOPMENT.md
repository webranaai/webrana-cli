# Webrana Plugin Development Guide

> Created by: CIPHER (Team Beta)  
> Version: 1.0.0

## Overview

Webrana supports extensible plugins via WebAssembly (WASM). Plugins can add new skills, commands, and capabilities to the agent without modifying core code.

## Quick Start

### 1. Create Plugin Directory

```bash
mkdir -p plugins/my-plugin
cd plugins/my-plugin
```

### 2. Create Manifest

Create `manifest.yaml`:

```yaml
id: my-plugin
name: My Plugin
version: 1.0.0
description: A custom Webrana plugin

author:
  name: Your Name
  email: you@example.com

plugin_type: wasm
min_webrana_version: "0.3.0"

permissions:
  - fs:read

skills:
  - name: my_skill
    description: Does something useful
    input_schema:
      type: object
      properties:
        input:
          type: string
      required: [input]
    requires_confirmation: false

entry_point: plugin.wat
```

### 3. Create WASM Module

Create `plugin.wat` (WebAssembly Text Format):

```wat
(module
  (func (export "my_skill") (result i32)
    i32.const 42
  )
)
```

### 4. Install Plugin

```bash
cp -r plugins/my-plugin ~/.config/webrana/plugins/
```

---

## Manifest Reference

### Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique plugin identifier (kebab-case) |
| `name` | string | Human-readable name |
| `version` | string | Semantic version (X.Y.Z) |
| `description` | string | Brief description |
| `plugin_type` | string | Must be `wasm` |
| `entry_point` | string | WASM/WAT filename |

### Optional Fields

| Field | Type | Description |
|-------|------|-------------|
| `author` | object | `{name, email}` |
| `min_webrana_version` | string | Minimum compatible version |
| `permissions` | array | Required permissions |
| `skills` | array | Exported skill definitions |
| `config` | object | Plugin configuration schema |

### Permissions

```yaml
permissions:
  - fs:read      # Read files
  - fs:write     # Write files
  - shell:execute # Run shell commands
  - net:request  # HTTP requests
  - env:read     # Read environment variables
  - git:access   # Git operations
  - llm:access   # Call LLM APIs
```

### Skill Definition

```yaml
skills:
  - name: skill_name          # Maps to WASM export
    description: What it does
    input_schema:             # JSON Schema
      type: object
      properties:
        param1:
          type: string
          description: Parameter description
      required: [param1]
    requires_confirmation: false  # Prompt user before execution
```

---

## WASM Development

### Supported Formats

| Format | Extension | Description |
|--------|-----------|-------------|
| WAT | `.wat` | WebAssembly Text (human-readable) |
| WASM | `.wasm` | WebAssembly Binary (compiled) |

### Function Conventions

**Export Pattern:** Skill name maps directly to WASM export.

```wat
;; Skill "greet" → export "greet"
(func (export "greet") (result i32)
  i32.const 42
)
```

**Return Values:**
- `i32` - Integer result (JSON: `{"result": <value>}`)
- `i64` - Long integer
- `f32/f64` - Floating point

### Example: Calculator Plugin

```wat
(module
  ;; Add two numbers
  (func (export "add") (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.add
  )

  ;; Multiply two numbers
  (func (export "multiply") (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.mul
  )

  ;; Subtract
  (func (export "subtract") (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.sub
  )
)
```

### Compiling WAT to WASM

```bash
# Using wabt (WebAssembly Binary Toolkit)
wat2wasm plugin.wat -o plugin.wasm

# Or use Webrana's built-in WAT support (no compilation needed)
```

---

## Plugin Discovery

Webrana searches for plugins in these locations (in order):

1. `./.webrana/plugins/` - Project-local plugins
2. `~/.config/webrana/plugins/` - User plugins
3. `/usr/share/webrana/plugins/` - System plugins

### Directory Structure

```
plugins/
└── my-plugin/
    ├── manifest.yaml    # Required: Plugin metadata
    ├── plugin.wat       # Required: WASM source (or .wasm)
    ├── README.md        # Optional: Documentation
    └── config.yaml      # Optional: Default config
```

---

## Advanced Topics

### Memory Management

WASM plugins have isolated memory (default 64MB limit).

```wat
(module
  ;; Declare 1 page (64KB) of memory
  (memory (export "memory") 1)
  
  ;; Store data at offset 0
  (func (export "store") (param $value i32)
    i32.const 0
    local.get $value
    i32.store
  )
  
  ;; Load data from offset 0
  (func (export "load") (result i32)
    i32.const 0
    i32.load
  )
)
```

### String Handling

For string parameters, use memory + pointer conventions:

```wat
(module
  (memory (export "memory") 1)
  
  ;; Allocate space and return pointer
  (func (export "alloc") (param $size i32) (result i32)
    ;; Simple bump allocator at offset 1024
    i32.const 1024
  )
  
  ;; Get string length at pointer
  (func (export "strlen") (param $ptr i32) (result i32)
    ;; Implementation...
    i32.const 0
  )
)
```

### Error Handling

Return negative values to indicate errors:

```wat
(func (export "divide") (param $a i32) (param $b i32) (result i32)
  ;; Check for division by zero
  local.get $b
  i32.eqz
  if (result i32)
    i32.const -1  ;; Error: division by zero
  else
    local.get $a
    local.get $b
    i32.div_s
  end
)
```

---

## Security

### Sandboxing

- Plugins run in isolated WASM sandbox
- No direct filesystem/network access
- Must request permissions via manifest
- Memory limited to declared pages

### Permission Enforcement

```yaml
# Plugin requesting file read
permissions:
  - fs:read

# Webrana validates before execution:
# ✓ fs:read allows ReadFile skill
# ✗ fs:read does NOT allow WriteFile
```

### Best Practices

1. Request minimal permissions
2. Validate all inputs
3. Handle errors gracefully
4. Document security implications
5. Use confirmation for destructive actions

---

## Testing Plugins

### Manual Testing

```bash
# Load and test plugin
webrana plugins list
webrana plugins test my-plugin

# Execute specific skill
webrana run --plugin my-plugin --skill add --args '{"a": 5, "b": 3}'
```

### Integration Testing

```rust
#[test]
fn test_my_plugin() {
    use wasmtime::{Engine, Module, Store, Linker};
    
    let engine = Engine::default();
    let wat = include_str!("../plugins/my-plugin/plugin.wat");
    let module = Module::new(&engine, wat).unwrap();
    
    let mut store = Store::new(&engine, ());
    let linker = Linker::new(&engine);
    let instance = linker.instantiate(&mut store, &module).unwrap();
    
    let add = instance.get_typed_func::<(i32, i32), i32>(&mut store, "add").unwrap();
    assert_eq!(add.call(&mut store, (2, 3)).unwrap(), 5);
}
```

---

## Sample Plugins

### hello-world
Basic demonstration plugin with `greet`, `add`, `multiply` functions.

Location: `plugins/hello-world/`

### file-analyzer (coming soon)
Analyze file contents, count lines, detect encoding.

### git-helper (coming soon)
Git operations: status, diff summary, branch info.

---

## Troubleshooting

### Plugin Not Found
```
Error: Plugin 'my-plugin' not found
```
**Solution:** Ensure plugin is in a discovery path and has valid `manifest.yaml`.

### WASM Compilation Error
```
Error: Failed to compile WAT module
```
**Solution:** Validate WAT syntax at https://webassembly.github.io/wabt/demo/wat2wasm/

### Permission Denied
```
Error: Plugin lacks required permission: fs:write
```
**Solution:** Add required permission to `manifest.yaml`.

### Function Not Found
```
Error: Function 'my_func' not found
```
**Solution:** Ensure function is exported: `(func (export "my_func") ...)`

---

## Resources

- [WebAssembly Specification](https://webassembly.github.io/spec/)
- [WAT Reference](https://developer.mozilla.org/en-US/docs/WebAssembly/Understanding_the_text_format)
- [wasmtime Documentation](https://docs.wasmtime.dev/)
- [Webrana GitHub](https://github.com/webranaai/webrana-cli)
