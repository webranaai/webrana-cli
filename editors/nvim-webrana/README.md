# Webrana CLI for Neovim

AI-powered autonomous coding assistant for Neovim.

## Installation

### Using lazy.nvim

```lua
{
    "webranaai/webrana-cli",
    config = function()
        require("webrana").setup({
            executable = "webrana",
            auto_mode = false,
            max_iterations = 10,
        })
    end,
}
```

### Using packer.nvim

```lua
use {
    "webranaai/webrana-cli",
    config = function()
        require("webrana").setup()
    end,
}
```

## Requirements

- Neovim 0.8+
- [Webrana CLI](https://github.com/webranaai/webrana-cli) installed
- API key configured (OpenAI or Anthropic)

## Commands

| Command | Description |
|---------|-------------|
| `:WebranaChat [msg]` | Start chat session |
| `:WebranaRun <task>` | Run autonomous task |
| `:WebranaExplain` | Explain selected code |
| `:WebranaFix` | Fix selected code |
| `:WebranaTest` | Generate tests for file |
| `:WebranaScan` | Scan for secrets |

## Keymaps

Default keymaps (leader = `\`):

| Keymap | Mode | Action |
|--------|------|--------|
| `<leader>wc` | Normal | Start chat |
| `<leader>we` | Visual | Explain selection |
| `<leader>wf` | Visual | Fix selection |
| `<leader>wt` | Normal | Generate tests |
| `<leader>ws` | Normal | Scan secrets |

## Configuration

```lua
require("webrana").setup({
    -- Path to webrana executable
    executable = "webrana",
    
    -- Skip confirmation prompts
    auto_mode = false,
    
    -- Max iterations for autonomous tasks
    max_iterations = 10,
})
```

## Usage Examples

### Chat

```vim
:WebranaChat How do I implement a binary search?
```

### Run Task

```vim
:WebranaRun Add error handling to all database queries
```

### Visual Selection

1. Select code in visual mode (`v` or `V`)
2. Run `:WebranaExplain` or `:WebranaFix`

## License

MIT
