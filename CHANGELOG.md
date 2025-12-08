# Changelog

All notable changes to Webrana CLI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2024-12-08

### Added

#### Intelligence & RAG (Sprint 5.2)
- **Embeddings Module** - Vector embeddings with OpenAI provider
  - `src/embeddings/provider.rs` - OpenAI and Mock providers
  - `src/embeddings/store.rs` - In-memory vector store with persistence
  - Cosine similarity search with threshold filtering
- **Semantic Search** - Code-aware search across codebase
  - `webrana search <query>` - Search indexed codebase
  - `webrana index` - Index directory for search
  - Text chunking with overlap for better context
- **RAG Context Builder** - Retrieval-augmented generation
  - Automatic context injection into LLM prompts
  - Configurable top_k and relevance threshold
- **Qdrant Integration** (optional)
  - `--features qdrant` for persistent vector storage
  - Collection management and filtered search

#### Security Hardening (Sprint 5.3)
- **Audit Logger** - Comprehensive operation logging
  - Event types: Command, File, LLM, Security
  - Severity levels with file/memory storage
  - Automatic sensitive data redaction
- **Secret Scanner** - Detect credentials in code
  - `webrana scan` - Scan for secrets
  - 25+ secret types (API keys, tokens, passwords)
  - CI/CD integration with `--fail-on-secrets`
- **Rate Limiter** - Token bucket algorithm
  - Per-operation rate limiting (API, LLM, File, Commands)
  - Burst allowance and configurable windows

#### Plugin Ecosystem (Sprint 5.4)
- **Plugin Manager** - Install and manage plugins
  - `webrana plugin list` - List installed plugins
  - `webrana plugin install <path>` - Install from local
  - `webrana plugin uninstall/enable/disable/info`
- **Example Plugins** - Reference implementations
  - `hello-plugin` (WASM) - Rust-based demo
  - `git-stats` (Script) - Shell-based demo

#### Platform Polish (Sprint 5.5)
- **Version Command** - `webrana version` with build info
- **Doctor Command** - `webrana doctor` system check
- **Warning Cleanup** - Reduced from 20+ to 3 warnings

### Changed
- Test count increased: 76 -> 126 tests
- Improved module organization
- Better error messages

### Technical
- New modules: embeddings, audit, secrets, rate_limit, rag
- Optional features: qdrant, tui
- Dependency: lazy_static for global instances

---

## [0.3.0-alpha] - 2024-12-07

### BREAKING CHANGES
- **Rust 1.80+ Required** - wasmtime dependency requires newer Rust version

### Added
- **WASM Plugin System** - Full WebAssembly plugin runtime with wasmtime 27
  - WAT (WebAssembly Text) format support
  - Plugin manifest (YAML) with skill definitions
  - Sandboxed execution environment
- **Sample Plugins** - 3 ready-to-use plugins:
  - `hello-world` - Basic demo (greet, add, multiply)
  - `calculator` - Math operations (add, subtract, multiply, divide, factorial, fibonacci, power, abs, max, min)
  - `text-utils` - String utilities (length, to_upper, to_lower, reverse, is_palindrome, count_chars)
- **Plugin Developer Guide** - Comprehensive documentation at `docs/PLUGIN_DEVELOPMENT.md`
- **76 Tests** - Expanded test suite across 8 test files:
  - `cli_test.rs` - 5 CLI integration tests
  - `config_test.rs` - 9 configuration tests
  - `llm_test.rs` - 12 LLM module tests
  - `memory_test.rs` - 12 context/memory tests
  - `plugin_test.rs` - 12 plugin system tests
  - `security_test.rs` - 4 security validation tests
  - `skills_test.rs` - 17 skills system tests
  - Unit tests - 5 core module tests

### Security
- **3-Layer Security System**:
  - Layer 1: `ALLOWED_COMMANDS` whitelist in shell.rs
  - Layer 2: `CommandRisk` assessment (Low/Medium/High/Critical)
  - Layer 3: Output sanitization (secrets, credentials, paths)
- **InputSanitizer** - Centralized input validation
- **ConfirmationPrompt** - User confirmation for risky operations
- Security integration in `file_ops.rs` (ReadFile, WriteFile)

### Changed
- Dockerfile updated to Rust 1.82
- Code formatting standardized with `cargo fmt`
- Improved error handling in plugin runtime

### Infrastructure
- CI workflow with multi-platform builds (Linux, macOS, Windows)
- Release workflow with automated artifacts
- Docker multi-arch support (amd64, arm64)

### Contributors
- **SENTINEL** - Security integration
- **CIPHER** - WASM runtime & plugins
- **VALIDATOR** - Test expansion
- **ATLAS** - CI/CD validation

---

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

[0.3.0-alpha]: https://github.com/webranaai/webrana-cli/releases/tag/v0.3.0-alpha
[0.3.0-beta]: https://github.com/webranaai/webrana-cli/releases/tag/v0.3.0-beta
[0.2.0]: https://github.com/webranaai/webrana-cli/releases/tag/v0.2.0
[0.1.0]: https://github.com/webranaai/webrana-cli/releases/tag/v0.1.0
