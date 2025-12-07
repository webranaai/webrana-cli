# Hello Plugin

A simple example plugin demonstrating the Webrana plugin system.

## Features

- `greet` - Greet a user by name
- `count_files` - Count files in a directory

## Installation

```bash
webrana plugin install ./examples/plugins/hello-plugin
```

## Usage

Once installed, the skills become available:

```
> greet John
Hello, John! Welcome to Webrana CLI.

> count_files ./src
Found 42 files in ./src
```

## Building from Source

This plugin uses WebAssembly. To build:

```bash
cd examples/plugins/hello-plugin
cargo build --target wasm32-wasi --release
cp target/wasm32-wasi/release/hello_plugin.wasm plugin.wasm
```

## Plugin Structure

```
hello-plugin/
├── plugin.yaml      # Plugin manifest
├── plugin.wasm      # Compiled WASM binary
├── src/
│   └── lib.rs       # Plugin source code
├── Cargo.toml       # Rust dependencies
└── README.md        # This file
```

## Manifest Reference

See `plugin.yaml` for the full manifest format including:
- Plugin metadata (id, name, version)
- Author information
- Required permissions
- Skill definitions with JSON Schema
