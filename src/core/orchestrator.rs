use anyhow::Result;
use std::io::{self, Write};
use colored::Colorize;

use crate::config::Settings;
use crate::llm::{LlmClient, Message};
use crate::memory::Context;
use crate::skills::SkillRegistry;
use crate::ui::Console;

pub struct Orchestrator {
    settings: Settings,
    llm: LlmClient,
    context: Context,
    skills: SkillRegistry,
    console: Console,
    auto_mode: bool,
}

impl Orchestrator {
    pub async fn new(settings: Settings, auto_mode: bool) -> Result<Self> {
        let llm = LlmClient::new(&settings)?;
        let context = Context::new();
        let skills = SkillRegistry::new();
        let console = Console::new();

        Ok(Self {
            settings,
            llm,
            context,
            skills,
            console,
            auto_mode,
        })
    }

    pub async fn chat(&self, message: &str) -> Result<()> {
        self.console.user_message(message);

        let agent = self.settings.get_agent(&self.settings.default_agent)
            .expect("Default agent not found");

        println!("\n{} {}", 
            format!("[{}]", agent.name).green().bold(),
            "━".repeat(50).dimmed()
        );

        let response = self.llm.chat_with_tools(
            &agent.system_prompt,
            self.context.get_messages(),
            message,
            &self.skills,
        ).await?;

        // Execute any tool calls
        for tool_call in &response.tool_calls {
            println!("\n{} {}", 
                "[TOOL]".magenta(),
                tool_call.name.cyan()
            );

            let result = self.skills.execute(
                &tool_call.name,
                &tool_call.arguments,
                &self.settings,
            ).await;

            match result {
                Ok(output) => println!("{}", output.dimmed()),
                Err(e) => println!("{}", format!("Error: {}", e).red()),
            }
        }

        Ok(())
    }

    pub async fn repl(&self) -> Result<()> {
        self.console.info("Starting interactive mode. Type 'exit' to quit.\n");
        self.console.info(&format!("Model: {} | Agent: {}\n", 
            self.settings.default_model.cyan(),
            self.settings.default_agent.cyan()
        ));

        let agent = self.settings.get_agent(&self.settings.default_agent)
            .expect("Default agent not found");

        let mut history: Vec<Message> = Vec::new();

        loop {
            print!("\n{} ", "▶".cyan().bold());
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.is_empty() {
                continue;
            }

            match input.to_lowercase().as_str() {
                "exit" | "quit" | "q" => {
                    self.console.info("Goodbye!");
                    break;
                }
                "clear" | "reset" => {
                    history.clear();
                    self.console.info("Context cleared.");
                    continue;
                }
                "skills" => {
                    self.console.list_skills();
                    continue;
                }
                "agents" => {
                    self.console.list_agents(&self.settings);
                    continue;
                }
                "help" | "?" => {
                    self.print_help();
                    continue;
                }
                "history" => {
                    println!("\n{}", "Conversation History:".bold().underline());
                    for (i, msg) in history.iter().enumerate() {
                        let role = match msg.role {
                            crate::llm::Role::User => "USER".blue(),
                            crate::llm::Role::Assistant => "ASSISTANT".green(),
                            crate::llm::Role::System => "SYSTEM".yellow(),
                        };
                        let preview: String = msg.content.chars().take(100).collect();
                        println!("  {}. [{}] {}...", i + 1, role, preview);
                    }
                    continue;
                }
                _ => {}
            }

            println!("\n{} {}", 
                format!("[{}]", agent.name).green().bold(),
                "━".repeat(50).dimmed()
            );

            // Use the tool loop for multi-turn tool usage
            match self.llm.chat_with_tools_loop(
                &agent.system_prompt,
                &mut history,
                input,
                &self.skills,
            ).await {
                Ok(response) => {
                    // Response already streamed, just add to history
                    if !response.is_empty() {
                        history.push(Message::assistant(&response));
                    }
                }
                Err(e) => {
                    self.console.error(&format!("Error: {}", e));
                }
            }
        }

        Ok(())
    }

    pub async fn run_autonomous(&self, task: &str, max_iterations: usize, yolo: bool) -> Result<()> {
        let agent = self.settings.get_agent(&self.settings.default_agent)
            .expect("Default agent not found");

        let mut history: Vec<Message> = Vec::new();
        
        let enhanced_task = format!(
            "{}\n\nIMPORTANT: You are running in autonomous mode. \
            Work step by step until the task is FULLY complete. \
            After each action, evaluate progress and continue until done. \
            When finished, respond with 'TASK_COMPLETE' on a new line.",
            task
        );

        println!("\n{} {}", 
            "[TASK]".yellow().bold(),
            task.white()
        );
        println!("{}", "━".repeat(60).dimmed());

        for iteration in 1..=max_iterations {
            println!("\n{} {}/{}", 
                "[ITERATION]".blue().bold(),
                iteration.to_string().cyan(),
                max_iterations.to_string().dimmed()
            );

            let prompt = if iteration == 1 {
                enhanced_task.clone()
            } else {
                "Continue working on the task. If complete, respond with TASK_COMPLETE.".to_string()
            };

            match self.llm.chat_with_tools_loop(
                &agent.system_prompt,
                &mut history,
                &prompt,
                &self.skills,
            ).await {
                Ok(response) => {
                    if !response.is_empty() {
                        history.push(Message::assistant(&response));
                        
                        // Check for task completion
                        if response.contains("TASK_COMPLETE") {
                            println!("\n{}", "━".repeat(60).green());
                            println!("{} Task completed in {} iterations", 
                                "✓".green().bold(),
                                iteration.to_string().cyan()
                            );
                            return Ok(());
                        }
                    }
                }
                Err(e) => {
                    self.console.error(&format!("Error in iteration {}: {}", iteration, e));
                    if !yolo {
                        return Err(e);
                    }
                }
            }
        }

        println!("\n{}", "━".repeat(60).yellow());
        println!("{} Reached maximum iterations ({})", 
            "⚠".yellow().bold(),
            max_iterations
        );

        Ok(())
    }

    fn print_help(&self) {
        println!("\n{}", "WEBRANA COMMANDS".bold().underline());
        println!("{}", "─".repeat(40));
        println!("  {}  - Exit the REPL", "exit, quit, q".cyan());
        println!("  {}  - Clear conversation history", "clear, reset".cyan());
        println!("  {}      - List available skills", "skills".cyan());
        println!("  {}      - List available agents", "agents".cyan());
        println!("  {}     - Show conversation history", "history".cyan());
        println!("  {}    - Show this help", "help, ?".cyan());
        println!();
        println!("{}", "TIPS".bold().underline());
        println!("  • Just type your request and press Enter");
        println!("  • The agent can read/write files, run commands");
        println!("  • Use Ctrl+C to interrupt streaming");
        println!();
    }
}
