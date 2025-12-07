# 🦎 Webrana CLI

**Autonomous CLI Coding Agent** - Your AI-powered terminal companion for software development.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.80+-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-0.3.0--alpha-blue.svg)](https://github.com/webranaai/webrana-cli/releases)
[![Tests](https://img.shields.io/badge/tests-76%20passing-green.svg)](https://github.com/webranaai/webrana-cli/actions)

## Overview

Webrana CLI is an open-source, terminal-native AI coding assistant that works directly in your development environment. Built with Rust for performance and safety, it supports multiple LLM providers and comes with an extensible WASM plugin system.

### Key Features

- 🤖 **Multi-Model Support** - Claude, GPT-4, Ollama (local models)
- ⚡ **Streaming Responses** - Real-time output with SSE
- 🛠️ **16+ Built-in Skills** - File ops, Git, shell, codebase analysis
- 🔧 **Native Tool Calling** - Multi-turn execution with automatic context
- 🔒 **3-Layer Security** - Input validation, risk assessment, output sanitization
- 🐳 **Docker Ready** - Multi-platform containerization
- 🔌 **WASM Plugins** - WebAssembly plugin system with wasmtime
- 🏃 **Auto Mode** - Autonomous task execution with `webrana run`
- ✅ **76 Tests** - Comprehensive test coverage across 8 suites

## Installation

### From Source (Recommended)

```bash
# Clone the repository
git clone https://github.com/webranaai/webrana-cli.git
cd webrana-cli

# Build release binary
cargo build --release

# Install to PATH (optional)
cp target/release/webrana ~/.local/bin/
```

### Requirements

- **Rust 1.80.0 or newer** (required for wasmtime)
- One of: Anthropic API key, OpenAI API key, or Ollama running locally

## Quick Start

### 1. Configure API Key

```bash
# Set your preferred provider
export ANTHROPIC_API_KEY="your-key-here"
# or
export OPENAI_API_KEY="your-key-here"
# or run Ollama locally
```

Or create `~/.config/webrana/config.toml`:

```toml
[llm]
default_provider = "anthropic"

[llm.anthropic]
api_key = "your-key-here"
model = "claude-sonnet-4-20250514"

[llm.openai]
api_key = "your-key-here"
model = "gpt-4"

[llm.ollama]
base_url = "http://localhost:11434"
model = "llama3"
```

### 2. Start Chatting

```bash
# Interactive chat mode
webrana chat

# Or with a direct question
webrana chat "explain this codebase"
```

### 3. Auto Mode (Autonomous Execution)

```bash
# Let Webrana complete a task autonomously
webrana run "refactor the authentication module to use JWT"
```

## Commands

| Command | Description |
|---------|-------------|
| `webrana chat [message]` | Interactive AI chat with tool execution |
| `webrana run <task>` | Autonomous task execution |
| `webrana agents` | List available AI agents |
| `webrana skills` | List available skills |
| `webrana config` | Show/edit configuration |
| `webrana mcp` | Start MCP server |

## Built-in Skills

### File Operations
- `read_file` - Read file contents
- `write_file` - Write/create files
- `list_files` - List directory contents
- `search_files` - Search by pattern

### Git Operations
- `git_status` - Repository status
- `git_diff` - Show changes
- `git_log` - Commit history
- `git_add` - Stage files
- `git_commit` - Create commits
- `git_branch` - List/create branches
- `git_checkout` - Switch branches

### Code Operations
- `shell_exec` - Execute shell commands (with safety checks)
- `edit_file` - Search and replace editing
- `grep_codebase` - Search code patterns
- `extract_symbols` - Extract functions/classes

## Docker

```bash
# Development
docker-compose up webrana-dev

# Production
docker-compose -f docker-compose.yml up webrana
```

## WASM Plugin System

Webrana supports WebAssembly plugins for extensibility. Plugins run in a secure sandbox with wasmtime.

### Sample Plugins Included

| Plugin | Description | Functions |
|--------|-------------|-----------|
| `hello-world` | Demo plugin | greet, add, multiply |
| `calculator` | Math operations | add, subtract, multiply, divide, factorial, fibonacci |
| `text-utils` | String utilities | length, to_upper, to_lower, reverse, is_palindrome |

### Creating a Plugin

1. Create plugin directory: `~/.config/webrana/plugins/my-plugin/`
2. Add `manifest.yaml`:

```yaml
id: my-plugin
name: My Plugin
version: 1.0.0
plugin_type: wasm
entry_point: plugin.wat

skills:
  - name: my_function
    description: Does something useful
```

3. Create `plugin.wat` (WebAssembly Text):

```wat
(module
  (func (export "my_function") (result i32)
    i32.const 42
  )
)
```

See [docs/PLUGIN_DEVELOPMENT.md](docs/PLUGIN_DEVELOPMENT.md) for full guide.

## Architecture

```
webrana/
├── src/
│   ├── main.rs          # Entry point
│   ├── cli/             # Command handlers
│   ├── core/            # Orchestrator, safety
│   ├── llm/             # Provider implementations
│   ├── skills/          # Skill registry & implementations
│   ├── indexer/         # Codebase indexing
│   ├── plugins/         # WASM plugin runtime
│   └── tui/             # Terminal UI (optional)
├── plugins/             # Sample WASM plugins
├── agents/              # Agent definitions
├── config/              # Default configs
├── docs/                # Documentation
└── tests/               # 76 tests across 8 suites
```

## Configuration

Default config location: `~/.config/webrana/config.toml`

```toml
[general]
auto_mode_max_iterations = 10
confirm_dangerous_commands = true

[llm]
default_provider = "anthropic"
temperature = 0.7
max_tokens = 4096

[skills]
enabled = ["file_ops", "git_ops", "shell", "codebase"]

[security]
blocked_commands = ["rm -rf /", ":(){ :|:& };:"]
require_confirmation = ["sudo", "rm -rf"]
```

## Security

Webrana includes built-in security features:

- **Command Risk Assessment** - Flags dangerous commands before execution
- **Path Traversal Prevention** - Blocks `../` escape attempts
- **Credential Redaction** - Hides API keys in logs
- **Confirmation Prompts** - Asks before risky operations
- **Sandboxed Execution** - Restricted shell environment

## Contributing

Contributions welcome! Please read our contributing guidelines first.

```bash
# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run -- chat
```

## Roadmap

- [x] Multi-model streaming
- [x] Native tool calling
- [x] Git integration
- [x] WASM plugin system (wasmtime)
- [x] 3-layer security hardening
- [x] 76 tests across 8 suites
- [x] Multi-platform CI/CD
- [ ] Persistent memory (SQLite)
- [ ] RAG with semantic search
- [ ] MCP client support
- [ ] VS Code extension

## License

MIT License - see [LICENSE](LICENSE) for details.

## Credits

Built with ❤️ by the Webrana Team

**AI Development Team:**
- NEXUS (CTO/Lead Architect)
- FORGE (Senior Engineer)
- SYNAPSE (AI/ML Specialist)
- COMPASS (Product Analyst)
- SCOUT (Research Lead)
- ATLAS (DevOps Lead)
- CIPHER (Plugin Developer)
- SENTINEL (Security Engineer)
- VALIDATOR (QA Engineer)

---

**Star ⭐ this repo if you find it useful!**
