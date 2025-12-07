use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "webrana")]
#[command(author = "Webrana Team")]
#[command(version = "0.3.0")]
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
}

#[derive(Subcommand)]
pub enum McpCommands {
    /// Start the MCP server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
}
