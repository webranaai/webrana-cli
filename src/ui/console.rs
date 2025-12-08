use colored::Colorize;

use crate::config::Settings;
use crate::skills::SkillRegistry;

pub struct Console;

impl Console {
    pub fn new() -> Self {
        Self
    }

    pub fn banner(&self) {
        let version = env!("CARGO_PKG_VERSION");
        
        println!(
            r#"
{}
{}
{}
{}
{}
{}
{}
{}
"#,
            "██╗    ██╗███████╗██████╗ ██████╗  █████╗ ███╗   ██╗ █████╗ ".bright_cyan(),
            "██║    ██║██╔════╝██╔══██╗██╔══██╗██╔══██╗████╗  ██║██╔══██╗".bright_cyan(),
            "██║ █╗ ██║█████╗  ██████╔╝██████╔╝███████║██╔██╗ ██║███████║".bright_cyan(),
            "██║███╗██║██╔══╝  ██╔══██╗██╔══██╗██╔══██║██║╚██╗██║██╔══██║".bright_cyan(),
            "╚███╔███╔╝███████╗██████╔╝██║  ██║██║  ██║██║ ╚████║██║  ██║".bright_cyan(),
            format!(" ╚══╝╚══╝ ╚══════╝╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝  v{}", version).bright_cyan(),
            "".normal(),
            "                    Ready When You Are".dimmed(),
        );
    }

    pub fn info(&self, message: &str) {
        println!("{} {}", "[INFO]".blue(), message);
    }

    pub fn warn(&self, message: &str) {
        println!("{} {}", "[WARN]".yellow(), message);
    }

    pub fn error(&self, message: &str) {
        println!("{} {}", "[ERROR]".red(), message);
    }

    pub fn success(&self, message: &str) {
        println!("{} {}", "[OK]".green(), message);
    }

    pub fn user_message(&self, message: &str) {
        println!("\n{} {}", "▶".cyan().bold(), message);
    }

    pub fn agent_message(&self, agent: &str, message: &str) {
        println!(
            "\n{} {}\n{}",
            format!("[{}]", agent).green().bold(),
            "━".repeat(50).dimmed(),
            message
        );
    }

    pub fn tool_result(&self, tool: &str, result: &str) {
        println!(
            "\n{} {}\n{}",
            format!("[TOOL:{}]", tool).magenta(),
            "─".repeat(40).dimmed(),
            result.dimmed()
        );
    }

    pub fn list_agents(&self, settings: &Settings) {
        println!("\n{}", "AVAILABLE AGENTS".bold().underline());
        println!("{}", "─".repeat(50));
        for (key, agent) in &settings.agents {
            println!(
                "  {} {} {}",
                "•".cyan(),
                key.cyan().bold(),
                format!("({})", agent.model).dimmed()
            );
            println!("    {}", agent.description);
        }
    }

    pub fn list_skills(&self) {
        let registry = SkillRegistry::new();
        let skills = registry.list();

        println!("\n{}", "AVAILABLE SKILLS".bold().underline());
        println!("{}", "─".repeat(50));

        // Group skills by category
        let file_skills: Vec<_> = skills
            .iter()
            .filter(|s| {
                s.name.contains("file") || s.name.contains("search") || s.name.contains("list")
            })
            .collect();

        let git_skills: Vec<_> = skills
            .iter()
            .filter(|s| s.name.starts_with("git_"))
            .collect();

        let other_skills: Vec<_> = skills
            .iter()
            .filter(|s| {
                !s.name.contains("file")
                    && !s.name.contains("search")
                    && !s.name.contains("list")
                    && !s.name.starts_with("git_")
            })
            .collect();

        if !file_skills.is_empty() {
            println!("\n  {}", "File Operations:".yellow());
            for skill in file_skills {
                self.print_skill(skill);
            }
        }

        if !git_skills.is_empty() {
            println!("\n  {}", "Git Operations:".yellow());
            for skill in git_skills {
                self.print_skill(skill);
            }
        }

        if !other_skills.is_empty() {
            println!("\n  {}", "System:".yellow());
            for skill in other_skills {
                self.print_skill(skill);
            }
        }
        println!();
    }

    fn print_skill(&self, skill: &crate::skills::SkillDefinition) {
        let confirm = if skill.requires_confirmation {
            " ⚠".yellow().to_string()
        } else {
            String::new()
        };
        println!(
            "    {} {}{}",
            skill.name.cyan(),
            format!("- {}", skill.description).dimmed(),
            confirm
        );
    }

    pub fn show_config(&self, settings: &Settings) {
        println!("\n{}", "CONFIGURATION".bold().underline());
        println!("{}", "─".repeat(50));

        println!("\n  {}", "Models:".yellow());
        for (key, model) in &settings.models {
            let is_default = key == &settings.default_model;
            let marker = if is_default { "→ " } else { "  " };
            println!(
                "  {}{} {} {}",
                marker.green(),
                key.cyan().bold(),
                format!("({})", model.provider).dimmed(),
                model.model.dimmed()
            );
        }

        println!(
            "\n  {} {}",
            "Default Model:".yellow(),
            settings.default_model.green()
        );
        println!(
            "  {} {}",
            "Default Agent:".yellow(),
            settings.default_agent.green()
        );

        println!("\n  {}", "Safety Settings:".yellow());
        println!(
            "    Confirm file writes: {}",
            if settings.safety.confirm_file_write {
                "yes".green()
            } else {
                "no".red()
            }
        );
        println!(
            "    Confirm shell commands: {}",
            if settings.safety.confirm_shell_execute {
                "yes".green()
            } else {
                "no".red()
            }
        );

        if let Ok(path) = Settings::config_path() {
            println!(
                "\n  {} {}",
                "Config file:".yellow(),
                path.display().to_string().dimmed()
            );
        }
        println!();
    }

    pub fn confirm(&self, message: &str) -> bool {
        print!("{} {} [y/N]: ", "[CONFIRM]".yellow().bold(), message);
        std::io::Write::flush(&mut std::io::stdout()).ok();

        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_ok() {
            let input = input.trim().to_lowercase();
            return input == "y" || input == "yes";
        }
        false
    }
}

impl Default for Console {
    fn default() -> Self {
        Self::new()
    }
}
