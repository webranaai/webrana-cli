# Webrana CLI

Autonomous CLI Coding Agent - Your AI-powered terminal companion for software development.

## Overview

Webrana CLI is an open-source, terminal-native AI coding assistant that works directly in your development environment. Built with Rust for performance and safety, it supports multiple LLM providers and comes with an extensible WASM plugin system.

### Key Features

- **Built-in Model** - Use without API key via api.webrana.id (50 req/day free)
- **Multi-Model Support** - Claude, GPT-4, Groq, Gemini, Ollama
- **Streaming Responses** - Real-time output with SSE
- **16+ Built-in Skills** - File ops, Git, shell, codebase analysis
- **Native Tool Calling** - Multi-turn execution with automatic context
- **3-Layer Security** - Input validation, risk assessment, output sanitization
- **Docker Ready** - Multi-platform containerization
- **WASM Plugins** - WebAssembly plugin system with wasmtime
- **Auto Mode** - Autonomous task execution with `webrana run`
- **Crew System** - Custom AI personas for specialized tasks

## Installation

### Via Cargo

```bash
cargo install webrana
```

### Via Homebrew (macOS)

```bash
brew tap webrana/tap
brew install webrana
```

### Via Docker

```bash
docker pull webrana/webrana-cli:latest
docker run -it webrana/webrana-cli
```

### From Source

```bash
git clone https://github.com/webranaai/webrana-cli.git
cd webrana-cli
cargo build --release
cp target/release/webrana ~/.local/bin/
```

### Requirements

- Rust 1.80.0 or newer (for building from source)
- Optional: API keys for Anthropic, OpenAI, or Groq

## Quick Start

### Option 1: Use Built-in Model (No API Key Required)

```bash
# Just start chatting - auto-registers with Webrana API
webrana chat "explain this codebase"

# Check your usage
webrana status
```

### Option 2: Use Your Own API Key

```bash
export ANTHROPIC_API_KEY="your-key-here"
# or
export OPENAI_API_KEY="your-key-here"

webrana chat "help me refactor this code"
```

## Commands

| Command | Description |
|---------|-------------|
| `webrana` | Start interactive REPL |
| `webrana chat [message]` | Chat with optional initial message |
| `webrana run <task>` | Autonomous task execution |
| `webrana status` | Check API usage (requests, tokens) |
| `webrana login` | Re-register device with API |
| `webrana logout` | Clear stored credentials |
| `webrana agents` | List available AI agents |
| `webrana skills` | List available skills |
| `webrana config` | Show configuration |
| `webrana crew` | Manage custom AI personas |
| `webrana plugin` | Manage WASM plugins |
| `webrana doctor` | Check system requirements |
| `webrana version` | Show version info |

## Built-in Skills

### File Operations
- `read_file` - Read file contents
- `write_file` - Write/create files  
- `edit_file` - Search and replace editing
- `list_files` - List directory contents
- `search_files` - Search by pattern

### Git Operations
- `git_status` - Repository status
- `git_diff` - Show changes
- `git_log` - Commit history
- `git_add` - Stage files
- `git_commit` - Create commits
- `git_branch` - List/create branches

### Code Operations
- `shell_exec` - Execute shell commands (with safety checks)
- `grep_codebase` - Search code patterns
- `extract_symbols` - Extract functions/classes

## Configuration

Config location: `~/.config/webrana/config.toml`

```toml
[llm]
default_provider = "anthropic"
temperature = 0.7
max_tokens = 4096

[llm.anthropic]
api_key = "sk-ant-..."
model = "claude-sonnet-4-20250514"

[llm.openai]
api_key = "sk-..."
model = "gpt-4"

[security]
confirm_dangerous_commands = true
blocked_commands = ["rm -rf /"]
```

## Webrana API

Webrana CLI includes a built-in model via api.webrana.id, allowing you to use the CLI without your own API keys.

### Free Tier
- 50 requests per day
- 100,000 tokens per day
- Powered by Groq (llama-3.3-70b)

### Commands
```bash
webrana status   # Check usage
webrana login    # Re-register device
webrana logout   # Clear credentials
```

Credentials are stored in `~/.config/webrana/credentials.json`

## WASM Plugin System

Webrana supports WebAssembly plugins for extensibility.

### Sample Plugins

| Plugin | Description |
|--------|-------------|
| hello-world | Demo plugin |
| calculator | Math operations |
| text-utils | String utilities |

### Creating a Plugin

1. Create directory: `~/.config/webrana/plugins/my-plugin/`
2. Add `manifest.yaml` and `plugin.wat`

See [docs/PLUGIN_DEVELOPMENT.md](docs/PLUGIN_DEVELOPMENT.md) for details.

## Crew System

Create custom AI personas for specialized tasks:

```bash
# List templates
webrana crew templates

# Create from template
webrana crew create reviewer --template code-reviewer

# Use crew
webrana crew use reviewer
webrana chat "review this PR"

# Clear (use default)
webrana crew clear
```

## Architecture

```
webrana/
├── src/
│   ├── main.rs          # Entry point
│   ├── cli/             # Command handlers
│   ├── core/            # Orchestrator, safety
│   ├── llm/             # Provider implementations
│   ├── skills/          # Skill registry
│   ├── plugins/         # WASM plugin runtime
│   └── crew/            # AI personas
├── agents/              # Agent definitions
├── config/              # Default configs
└── tests/               # Test suites
```

## Security

Built-in security features:

- Command Risk Assessment - Flags dangerous commands
- Path Traversal Prevention - Blocks escape attempts
- Credential Redaction - Hides API keys in logs
- Confirmation Prompts - Asks before risky operations

## Development

```bash
# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run -- chat

# Build release
cargo build --release
```

## Roadmap

- [x] Multi-model streaming
- [x] Native tool calling
- [x] Git integration  
- [x] WASM plugin system
- [x] Security hardening
- [x] Built-in model (api.webrana.id)
- [x] Crew system
- [x] Docker support
- [ ] Persistent memory
- [ ] RAG with semantic search
- [ ] VS Code extension

## License

MIT License - see [LICENSE](LICENSE) for details.

---

Built by the Webrana Team
