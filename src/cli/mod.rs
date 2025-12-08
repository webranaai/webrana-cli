use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "webrana")]
#[command(author = "Webrana Team")]
#[command(version = "0.4.0")]
#[command(about = "Autonomous CLI Agent - Think. Code. Execute.", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Enable auto mode (no confirmation prompts)
    #[arg(short, long, global = true)]
    pub auto: bool,

    /// Maximum iterations in auto mode (default: 10)
    #[arg(long, default_value = "10", global = true)]
    pub max_iterations: usize,

    /// Working directory for the agent
    #[arg(short = 'd', long, global = true)]
    pub workdir: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start a chat session with a message
    Chat {
        /// The message to send
        #[arg(required = true)]
        message: String,

        /// Enable auto mode for this chat
        #[arg(short, long)]
        auto: bool,
    },

    /// Run a task autonomously until completion
    Run {
        /// The task to execute
        #[arg(required = true)]
        task: String,

        /// Maximum iterations (default: 25)
        #[arg(short, long, default_value = "25")]
        max_iterations: usize,

        /// Skip dangerous operation confirmations
        #[arg(long)]
        yolo: bool,
    },

    /// List available agents
    Agents,

    /// List available skills
    Skills,

    /// Show current configuration
    Config,

    /// Crew management (custom AI personas)
    Crew {
        #[command(subcommand)]
        command: CrewCommands,
    },

    /// MCP server commands
    Mcp {
        #[command(subcommand)]
        command: McpCommands,
    },

    /// Launch Terminal User Interface
    Tui,

    /// Semantic search in codebase
    Search {
        /// Search query
        #[arg(required = true)]
        query: String,

        /// Directory to search in (default: current directory)
        #[arg(short, long)]
        dir: Option<String>,

        /// Number of results to return
        #[arg(short = 'n', long, default_value = "5")]
        top_k: usize,

        /// Index the codebase before searching
        #[arg(long)]
        index: bool,
    },

    /// Index codebase for semantic search
    Index {
        /// Directory to index (default: current directory)
        #[arg(short, long)]
        dir: Option<String>,
    },

    /// Scan for secrets and credentials in codebase
    Scan {
        /// Directory to scan (default: current directory)
        #[arg(long)]
        dir: Option<String>,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Minimum severity to report (low, medium, high, critical)
        #[arg(long, default_value = "low")]
        min_severity: String,

        /// Fail with exit code 1 if secrets found
        #[arg(long)]
        fail_on_secrets: bool,
    },

    /// Plugin management commands
    Plugin {
        #[command(subcommand)]
        command: PluginCommands,
    },

    /// Show version and build information
    Version,

    /// Check system requirements and configuration
    Doctor,

    /// Check for updates
    Update,
}

#[derive(Subcommand)]
pub enum PluginCommands {
    /// List installed plugins
    List,

    /// Install a plugin from local path
    Install {
        /// Path to plugin directory
        path: String,
    },

    /// Uninstall a plugin
    Uninstall {
        /// Plugin ID to uninstall
        plugin_id: String,
    },

    /// Enable a plugin
    Enable {
        /// Plugin ID to enable
        plugin_id: String,
    },

    /// Disable a plugin
    Disable {
        /// Plugin ID to disable
        plugin_id: String,
    },

    /// Show plugin info
    Info {
        /// Plugin ID
        plugin_id: String,
    },
}

#[derive(Subcommand)]
pub enum McpCommands {
    /// Start the MCP server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },

    /// List connected MCP servers
    List,

    /// Connect to an MCP server
    Connect {
        /// Server name
        name: String,

        /// Command to run
        command: String,

        /// Arguments for the command
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Disconnect from an MCP server
    Disconnect {
        /// Server name
        name: String,
    },

    /// List tools from a specific server
    Tools {
        /// Server name (optional, lists all if not specified)
        #[arg(short, long)]
        server: Option<String>,
    },

    /// Call an MCP tool
    Call {
        /// Tool name
        tool: String,

        /// JSON arguments
        #[arg(short, long, default_value = "{}")]
        args: String,
    },
}

#[derive(Subcommand)]
pub enum CrewCommands {
    /// List all crew members
    List,

    /// Create a new crew member
    Create {
        /// Crew ID (lowercase, no spaces)
        id: String,

        /// Display name
        #[arg(short, long)]
        name: Option<String>,

        /// Description
        #[arg(short, long)]
        description: Option<String>,

        /// System prompt (or use --from-template)
        #[arg(short, long)]
        prompt: Option<String>,

        /// Create from template (code-reviewer, bug-hunter, doc-writer, refactorer, test-engineer, security-auditor, devops-engineer)
        #[arg(short, long)]
        template: Option<String>,
    },

    /// Show crew member details
    Show {
        /// Crew ID
        id: String,
    },

    /// Delete a crew member
    Delete {
        /// Crew ID
        id: String,
    },

    /// Set active crew member
    Use {
        /// Crew ID to activate
        id: String,
    },

    /// Clear active crew (use default agent)
    Clear,

    /// Export crew to YAML
    Export {
        /// Crew ID
        id: String,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Import crew from YAML file
    Import {
        /// YAML file path
        file: String,
    },

    /// List available templates
    Templates,
}
