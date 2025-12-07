# Changelog

All notable changes to Webrana AI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0-beta] - 2024-12-07

### Added
- **Plugin System** - Extensible architecture with manifest support (YAML/TOML)
- **Security Module** - Input sanitization, command risk assessment, credential redaction
- **Docker Support** - Multi-stage Dockerfile and docker-compose configurations
- **CI/CD Pipeline** - GitHub Actions with cross-platform builds (Linux, macOS, Windows)
- **Test Suite** - 20 tests covering security, plugins, CLI, and core functionality
- **TUI Module** - Terminal UI framework (optional, requires Rust 1.80+)

### Changed
- Improved dependency management for Rust 1.75.0 compatibility
- Enhanced streaming response handling for all providers
- Better error messages and user feedback

### Security
- Command injection prevention
- Path traversal blocking
- Dangerous command confirmation prompts
- API key redaction in logs

## [0.2.0] - 2024-12-01

### Added
- **Multi-Model Support** - Claude (Anthropic), GPT-4 (OpenAI), Ollama (local)
- **Streaming Responses** - Real-time SSE streaming for all providers
- **Native Tool Calling** - Anthropic tool_use and OpenAI function_calling
- **Multi-turn Execution** - Automatic tool result injection (max 10 iterations)
- **Git Integration** - 7 skills: status, diff, log, add, commit, branch, checkout
- **Auto Mode** - Autonomous task execution with webrana run
- **Codebase Intelligence** - FileWalker, FileIndex, ProjectDetector
- **Edit Skills** - Search/replace editing, grep, symbol extraction

### Changed
- Enhanced NEXUS system prompt for coding tasks
- Improved orchestrator with streaming + tools integration

## [0.1.0] - 2024-11-15

### Added
- Initial release
- CLI framework with clap
- BYOK configuration (TOML-based)
- LLM provider abstraction trait
- 5 core skills: read_file, write_file, list_files, search_files, shell_exec
- Basic MCP server (JSON-RPC 2.0)

---

[0.3.0-beta]: https://github.com/webranaai/webrana-cli/releases/tag/v0.3.0-beta
[0.2.0]: https://github.com/webranaai/webrana-cli/releases/tag/v0.2.0
[0.1.0]: https://github.com/webranaai/webrana-cli/releases/tag/v0.1.0
