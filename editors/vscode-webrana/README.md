# Webrana CLI for VS Code

AI-powered autonomous coding assistant integrated into VS Code.

## Features

- **Chat** - Start a conversation with Webrana AI
- **Run Task** - Execute autonomous coding tasks
- **Explain** - Get explanations for selected code
- **Fix** - Automatically fix issues in selected code
- **Generate Tests** - Create unit tests for your code
- **Scan Secrets** - Detect credentials in your codebase

## Requirements

- [Webrana CLI](https://github.com/webranaai/webrana-cli) installed and in PATH
- API key configured (OpenAI or Anthropic)

## Installation

1. Install Webrana CLI:
   ```bash
   # Download from releases
   curl -sSL https://github.com/webranaai/webrana-cli/releases/latest/download/webrana-linux-x86_64 -o webrana
   chmod +x webrana
   sudo mv webrana /usr/local/bin/
   ```

2. Configure API key:
   ```bash
   export OPENAI_API_KEY=sk-...
   # or
   export ANTHROPIC_API_KEY=sk-ant-...
   ```

3. Install this extension from VS Code Marketplace

## Usage

### Keyboard Shortcuts

- `Ctrl+Shift+W` (Mac: `Cmd+Shift+W`) - Start chat

### Context Menu

Right-click on selected code to:
- Explain Selection
- Fix Selection
- Generate Tests

### Command Palette

Press `Ctrl+Shift+P` and type "Webrana" to see all commands:
- Webrana: Start Chat
- Webrana: Run Task
- Webrana: Explain Selection
- Webrana: Fix Selection
- Webrana: Generate Tests
- Webrana: Scan for Secrets

## Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `webrana.executablePath` | `webrana` | Path to CLI |
| `webrana.autoMode` | `false` | Skip confirmations |
| `webrana.maxIterations` | `10` | Max autonomous iterations |

## Development

```bash
cd editors/vscode-webrana
npm install
npm run compile
```

Press F5 in VS Code to launch extension development host.

## License

MIT
