// Allow dead code for modules prepared for future use
#![allow(dead_code)]

mod cli;
mod config;
mod core;
mod embeddings;
mod indexer;
mod llm;
mod mcp;
mod memory;
mod plugins;
mod skills;
mod tui;
mod ui;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::cli::{Cli, Commands};
use crate::config::Settings;
use crate::core::Orchestrator;
use crate::ui::Console;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let cli = Cli::parse();
    let settings = Settings::load()?;
    let console = Console::new();

    console.banner();

    // Change working directory if specified
    if let Some(workdir) = &cli.workdir {
        std::env::set_current_dir(workdir)?;
        console.info(&format!("Working directory: {}", workdir));
    }

    match cli.command {
        Some(Commands::Chat { message, auto }) => {
            let orchestrator = Orchestrator::new(settings, auto || cli.auto).await?;
            orchestrator.chat(&message).await?;
        }
        Some(Commands::Run {
            task,
            max_iterations,
            yolo,
        }) => {
            console.info(&format!(
                "🤖 Auto Mode: max {} iterations{}",
                max_iterations,
                if yolo { " (YOLO mode)" } else { "" }
            ));
            let orchestrator = Orchestrator::new(settings, true).await?;
            orchestrator
                .run_autonomous(&task, max_iterations, yolo)
                .await?;
        }
        Some(Commands::Agents) => {
            console.list_agents(&settings);
        }
        Some(Commands::Skills) => {
            console.list_skills();
        }
        Some(Commands::Config) => {
            console.show_config(&settings);
        }
        Some(Commands::Mcp { command }) => {
            use mcp::{McpClient, McpRegistry, McpServerConfig};
            use std::collections::HashMap;
            use std::sync::Mutex;

            // Lazy static for persistent registry across commands
            static MCP_REGISTRY: std::sync::OnceLock<Mutex<McpRegistry>> = std::sync::OnceLock::new();
            let registry = MCP_REGISTRY.get_or_init(|| Mutex::new(McpRegistry::new()));

            match command {
                cli::McpCommands::Serve { port } => {
                    console.info(&format!("Starting MCP server on port {}...", port));
                    mcp::server::start(port).await?;
                }
                cli::McpCommands::List => {
                    let reg = registry.lock().unwrap();
                    let servers = reg.connected_servers();
                    if servers.is_empty() {
                        console.info("No MCP servers connected");
                    } else {
                        println!("\nConnected MCP servers:\n");
                        for name in servers {
                            let info = reg.server_info(name).unwrap_or_else(|| "unknown".to_string());
                            println!("  {} - {}", name, info);
                        }
                    }
                }
                cli::McpCommands::Connect { name, command, args } => {
                    console.info(&format!("Connecting to MCP server '{}'...", name));
                    let config = McpServerConfig {
                        command,
                        args,
                        env: HashMap::new(),
                        enabled: true,
                    };
                    let mut reg = registry.lock().unwrap();
                    match reg.add_server(&name, &config) {
                        Ok(_) => {
                            let tools = reg.list_server_tools(&name).map(|t| t.len()).unwrap_or(0);
                            console.success(&format!("Connected to '{}' ({} tools available)", name, tools));
                        }
                        Err(e) => {
                            console.error(&format!("Failed to connect: {}", e));
                        }
                    }
                }
                cli::McpCommands::Disconnect { name } => {
                    let mut reg = registry.lock().unwrap();
                    match reg.remove_server(&name) {
                        Ok(_) => console.success(&format!("Disconnected from '{}'", name)),
                        Err(e) => console.error(&format!("Failed to disconnect: {}", e)),
                    }
                }
                cli::McpCommands::Tools { server } => {
                    let reg = registry.lock().unwrap();
                    let tools = if let Some(name) = server {
                        reg.list_server_tools(&name)
                            .map(|t| t.iter().map(|tool| (name.clone(), tool.clone())).collect())
                            .unwrap_or_default()
                    } else {
                        reg.list_all_tools()
                    };

                    if tools.is_empty() {
                        console.info("No tools available. Connect to an MCP server first.");
                    } else {
                        println!("\nAvailable MCP tools:\n");
                        for (server_name, tool) in tools {
                            println!("  {} (from {})", tool.name, server_name);
                            if let Some(desc) = &tool.description {
                                println!("    {}", desc);
                            }
                        }
                    }
                }
                cli::McpCommands::Call { tool, args } => {
                    let arguments: HashMap<String, serde_json::Value> = 
                        serde_json::from_str(&args).unwrap_or_default();
                    
                    let mut reg = registry.lock().unwrap();
                    match reg.call_tool(&tool, arguments) {
                        Ok(result) => {
                            for content in result.content {
                                match content {
                                    mcp::ToolContent::Text { text } => println!("{}", text),
                                    mcp::ToolContent::Image { data, mime_type } => {
                                        println!("[Image: {} bytes, {}]", data.len(), mime_type);
                                    }
                                    mcp::ToolContent::Resource { uri, .. } => {
                                        println!("[Resource: {}]", uri);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            console.error(&format!("Tool call failed: {}", e));
                        }
                    }
                }
            }
        }
        Some(Commands::Tui) => {
            tui::run_tui().await?;
        }
        Some(Commands::Search {
            query,
            dir,
            top_k,
            index,
        }) => {
            use skills::{SemanticSearch, SemanticSearchConfig};
            use std::path::Path;

            let search_dir = dir.as_deref().unwrap_or(".");
            let config = SemanticSearchConfig {
                top_k,
                ..Default::default()
            };

            // Check for API key
            let api_key = std::env::var("OPENAI_API_KEY").ok();
            
            let mut search = if let Some(key) = api_key {
                SemanticSearch::new(&key, config)
            } else {
                console.warn("OPENAI_API_KEY not set, using mock embeddings");
                SemanticSearch::new_mock(config)
            };

            if index {
                console.info(&format!("Indexing {}...", search_dir));
                let stats = search.index_directory(Path::new(search_dir)).await?;
                console.info(&format!(
                    "Indexed {} files, {} chunks ({} skipped, {} errors)",
                    stats.files, stats.chunks, stats.skipped, stats.errors
                ));
            }

            console.info(&format!("Searching for: {}", query));
            let results = search.search(&query).await?;

            if results.is_empty() {
                console.warn("No results found. Try indexing first with --index");
            } else {
                for (i, result) in results.iter().enumerate() {
                    println!("\n{}. {} (score: {:.3})", i + 1, result.id, result.score);
                    if let Some(file) = result.metadata.get("file") {
                        println!("   File: {}", file);
                    }
                    // Show snippet
                    let snippet: String = result.text.chars().take(200).collect();
                    println!("   {}", snippet.replace('\n', " "));
                }
            }
        }
        Some(Commands::Index { dir }) => {
            use skills::{SemanticSearch, SemanticSearchConfig};
            use std::path::Path;

            let search_dir = dir.as_deref().unwrap_or(".");
            let config = SemanticSearchConfig::default();

            let api_key = std::env::var("OPENAI_API_KEY").ok();
            
            let mut search = if let Some(key) = api_key {
                SemanticSearch::new(&key, config)
            } else {
                console.warn("OPENAI_API_KEY not set, using mock embeddings");
                SemanticSearch::new_mock(config)
            };

            console.info(&format!("Indexing {}...", search_dir));
            let stats = search.index_directory(Path::new(search_dir)).await?;
            console.info(&format!(
                "Done! Indexed {} files, {} chunks ({} skipped, {} errors)",
                stats.files, stats.chunks, stats.skipped, stats.errors
            ));
        }
        Some(Commands::Scan {
            dir,
            format,
            min_severity,
            fail_on_secrets,
        }) => {
            use core::{ScanSummary, ScannerConfig, SecretScanner, SecretSeverity};
            use std::path::Path;

            let scan_dir = dir.as_deref().unwrap_or(".");
            
            // Parse minimum severity
            let min_sev = match min_severity.to_lowercase().as_str() {
                "low" => SecretSeverity::Low,
                "medium" => SecretSeverity::Medium,
                "high" => SecretSeverity::High,
                "critical" => SecretSeverity::Critical,
                _ => {
                    console.error("Invalid severity. Use: low, medium, high, critical");
                    return Ok(());
                }
            };

            let config = ScannerConfig {
                min_severity: min_sev,
                ..Default::default()
            };

            let scanner = SecretScanner::new(config);
            
            console.info(&format!("Scanning {} for secrets...", scan_dir));
            
            let secrets = scanner.scan_directory(Path::new(scan_dir))?;
            let summary = ScanSummary::from_secrets(&secrets);

            if format == "json" {
                println!("{}", serde_json::to_string_pretty(&secrets)?);
            } else {
                if secrets.is_empty() {
                    console.success("No secrets detected!");
                } else {
                    println!("\n{} secrets found:\n", secrets.len());
                    
                    for secret in &secrets {
                        let severity_icon = match secret.severity {
                            SecretSeverity::Critical => "🔴 CRITICAL",
                            SecretSeverity::High => "🟠 HIGH",
                            SecretSeverity::Medium => "🟡 MEDIUM",
                            SecretSeverity::Low => "🟢 LOW",
                        };
                        
                        println!(
                            "{}: {}:{}\n   Type: {}\n   Match: {}\n",
                            severity_icon,
                            secret.file,
                            secret.line,
                            secret.secret_type.description(),
                            secret.matched_text
                        );
                    }

                    println!("Summary:");
                    println!("  Files with secrets: {}", summary.files_with_secrets);
                    println!("  Total secrets: {}", summary.total_secrets);
                    
                    for (severity, count) in &summary.by_severity {
                        println!("  {}: {}", severity, count);
                    }
                }
            }

            if fail_on_secrets && !secrets.is_empty() {
                std::process::exit(1);
            }
        }
        Some(Commands::Plugin { command }) => {
            use plugins::PluginManager;
            use std::path::Path;

            let mut manager = PluginManager::default_manager()?;

            match command {
                cli::PluginCommands::List => {
                    let plugins = manager.list();
                    if plugins.is_empty() {
                        console.info("No plugins installed");
                    } else {
                        println!("\nInstalled plugins:\n");
                        for plugin in plugins {
                            let status = if plugin.config.enabled { "enabled" } else { "disabled" };
                            println!(
                                "  {} v{} [{}]",
                                plugin.manifest.name,
                                plugin.manifest.version,
                                status
                            );
                            println!("    ID: {}", plugin.manifest.id);
                            println!("    {}\n", plugin.manifest.description);
                        }
                        let stats = manager.stats();
                        println!("Total: {} ({} enabled, {} disabled)", stats.total, stats.enabled, stats.disabled);
                    }
                }
                cli::PluginCommands::Install { path } => {
                    console.info(&format!("Installing plugin from {}...", path));
                    match manager.install_local(Path::new(&path)) {
                        Ok(plugins::InstallResult::Installed(manifest)) => {
                            console.success(&format!("Installed {} v{}", manifest.name, manifest.version));
                        }
                        Ok(plugins::InstallResult::Updated(manifest)) => {
                            console.success(&format!("Updated {} to v{}", manifest.name, manifest.version));
                        }
                        Ok(plugins::InstallResult::AlreadyInstalled(id)) => {
                            console.warn(&format!("Plugin {} is already installed", id));
                        }
                        Err(e) => {
                            console.error(&format!("Failed to install: {}", e));
                        }
                    }
                }
                cli::PluginCommands::Uninstall { plugin_id } => {
                    if manager.uninstall(&plugin_id)? {
                        console.success(&format!("Uninstalled {}", plugin_id));
                    } else {
                        console.error(&format!("Plugin {} not found", plugin_id));
                    }
                }
                cli::PluginCommands::Enable { plugin_id } => {
                    if manager.enable(&plugin_id)? {
                        console.success(&format!("Enabled {}", plugin_id));
                    } else {
                        console.error(&format!("Plugin {} not found", plugin_id));
                    }
                }
                cli::PluginCommands::Disable { plugin_id } => {
                    if manager.disable(&plugin_id)? {
                        console.success(&format!("Disabled {}", plugin_id));
                    } else {
                        console.error(&format!("Plugin {} not found", plugin_id));
                    }
                }
                cli::PluginCommands::Info { plugin_id } => {
                    if let Some(plugin) = manager.get(&plugin_id) {
                        println!("\nPlugin: {}", plugin.manifest.name);
                        println!("ID: {}", plugin.manifest.id);
                        println!("Version: {}", plugin.manifest.version);
                        println!("Author: {}", plugin.manifest.author.name);
                        println!("Type: {:?}", plugin.manifest.plugin_type);
                        println!("Status: {}", if plugin.config.enabled { "enabled" } else { "disabled" });
                        println!("\nDescription:\n  {}", plugin.manifest.description);
                        println!("\nPermissions:");
                        for perm in &plugin.manifest.permissions {
                            println!("  - {:?}", perm);
                        }
                        println!("\nSkills:");
                        for skill in &plugin.manifest.skills {
                            println!("  - {}: {}", skill.name, skill.description);
                        }
                    } else {
                        console.error(&format!("Plugin {} not found", plugin_id));
                    }
                }
            }
        }
        Some(Commands::Version) => {
            println!("Webrana CLI v{}", env!("CARGO_PKG_VERSION"));
            println!("Build: {}", if cfg!(debug_assertions) { "debug" } else { "release" });
            println!("Target: {}", std::env::consts::ARCH);
            println!("OS: {}", std::env::consts::OS);
            println!();
            println!("Features:");
            println!("  - WASM plugins: enabled");
            #[cfg(feature = "qdrant")]
            println!("  - Qdrant: enabled");
            #[cfg(not(feature = "qdrant"))]
            println!("  - Qdrant: disabled");
            #[cfg(feature = "tui")]
            println!("  - TUI: enabled");
            #[cfg(not(feature = "tui"))]
            println!("  - TUI: disabled");
        }
        Some(Commands::Doctor) => {
            println!("Webrana CLI - System Check\n");
            
            // Check config
            print!("Configuration... ");
            if settings.get_model(&settings.default_model).is_some() {
                println!("OK (model: {})", settings.default_model);
            } else {
                println!("WARN (no default model)");
            }

            // Check API keys
            print!("OpenAI API Key... ");
            if std::env::var("OPENAI_API_KEY").is_ok() {
                println!("OK");
            } else {
                println!("NOT SET");
            }

            print!("Anthropic API Key... ");
            if std::env::var("ANTHROPIC_API_KEY").is_ok() {
                println!("OK");
            } else {
                println!("NOT SET");
            }

            // Check git
            print!("Git... ");
            match std::process::Command::new("git").arg("--version").output() {
                Ok(output) => {
                    let version = String::from_utf8_lossy(&output.stdout);
                    println!("OK ({})", version.trim());
                }
                Err(_) => println!("NOT FOUND"),
            }

            // Check plugins directory
            print!("Plugins directory... ");
            let plugins_dir = directories::ProjectDirs::from("dev", "webrana", "webrana-cli")
                .map(|dirs| dirs.data_dir().join("plugins"));
            if let Some(dir) = plugins_dir {
                if dir.exists() {
                    println!("OK ({})", dir.display());
                } else {
                    println!("OK (will be created: {})", dir.display());
                }
            } else {
                println!("WARN (using .webrana/plugins)");
            }

            println!("\nAll checks complete.");
        }
        Some(Commands::Update) => {
            use core::updater::{check_for_updates, format_update_message, UpdateStatus};

            console.info("Checking for updates...");

            match check_for_updates().await {
                UpdateStatus::UpToDate => {
                    console.success(&format!("Webrana CLI v{} is up to date.", env!("CARGO_PKG_VERSION")));
                }
                UpdateStatus::UpdateAvailable { current, latest, url, .. } => {
                    println!("\nUpdate available!");
                    println!("  Current: v{}", current);
                    println!("  Latest:  v{}", latest);
                    println!("\nDownload: {}", url);
                    println!("\nTo update, download the latest release and replace the binary.");
                }
                UpdateStatus::CheckFailed(err) => {
                    console.error(&format!("Failed to check for updates: {}", err));
                }
            }
        }
        None => {
            let orchestrator = Orchestrator::new(settings, cli.auto).await?;
            orchestrator.repl().await?;
        }
    }

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "webrana=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}
