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
        Some(Commands::Mcp { command }) => match command {
            cli::McpCommands::Serve { port } => {
                console.info(&format!("Starting MCP server on port {}...", port));
                mcp::server::start(port).await?;
            }
        },
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
