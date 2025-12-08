//! Crew Persona Definition

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A Crew member - custom AI persona
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Crew {
    /// Unique identifier (lowercase, no spaces)
    pub id: String,
    
    /// Display name
    pub name: String,
    
    /// Short description
    pub description: String,
    
    /// System prompt that defines personality and behavior
    pub system_prompt: String,
    
    /// Configuration options
    #[serde(default)]
    pub config: CrewConfig,
    
    /// Permissions for this crew member
    #[serde(default)]
    pub permissions: CrewPermissions,
    
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
    
    /// Author info
    #[serde(default)]
    pub author: Option<String>,
    
    /// Version
    #[serde(default = "default_version")]
    pub version: String,
    
    /// Creation timestamp
    #[serde(default)]
    pub created_at: Option<String>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

/// Crew configuration options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CrewConfig {
    /// Preferred model (overrides default)
    #[serde(default)]
    pub model: Option<String>,
    
    /// Temperature for responses (0.0 - 2.0)
    #[serde(default)]
    pub temperature: Option<f32>,
    
    /// Max tokens for responses
    #[serde(default)]
    pub max_tokens: Option<u32>,
    
    /// Auto mode by default
    #[serde(default)]
    pub auto_mode: bool,
    
    /// Max iterations in auto mode
    #[serde(default = "default_max_iterations")]
    pub max_iterations: usize,
    
    /// Custom greeting message
    #[serde(default)]
    pub greeting: Option<String>,
}

fn default_max_iterations() -> usize {
    10
}

/// Permissions for crew member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrewPermissions {
    /// Allowed skills (empty = all allowed)
    #[serde(default)]
    pub allowed_skills: HashSet<String>,
    
    /// Denied skills (takes precedence over allowed)
    #[serde(default)]
    pub denied_skills: HashSet<String>,
    
    /// Can execute shell commands
    #[serde(default = "default_true")]
    pub shell_access: bool,
    
    /// Can read files
    #[serde(default = "default_true")]
    pub file_read: bool,
    
    /// Can write files
    #[serde(default = "default_true")]
    pub file_write: bool,
    
    /// Can access network
    #[serde(default = "default_true")]
    pub network_access: bool,
}

fn default_true() -> bool {
    true
}

impl Default for CrewPermissions {
    fn default() -> Self {
        Self {
            allowed_skills: HashSet::new(),
            denied_skills: HashSet::new(),
            shell_access: true,
            file_read: true,
            file_write: true,
            network_access: true,
        }
    }
}

impl Crew {
    /// Create a new crew member
    pub fn new(id: &str, name: &str, description: &str, system_prompt: &str) -> Self {
        Self {
            id: id.to_lowercase().replace(' ', "-"),
            name: name.to_string(),
            description: description.to_string(),
            system_prompt: system_prompt.to_string(),
            config: CrewConfig::default(),
            permissions: CrewPermissions::default(),
            tags: Vec::new(),
            author: None,
            version: "1.0.0".to_string(),
            created_at: Some(chrono_lite()),
        }
    }

    /// Check if a skill is allowed
    pub fn is_skill_allowed(&self, skill: &str) -> bool {
        // Denied takes precedence
        if self.permissions.denied_skills.contains(skill) {
            return false;
        }
        
        // If allowed list is empty, all are allowed
        if self.permissions.allowed_skills.is_empty() {
            return true;
        }
        
        self.permissions.allowed_skills.contains(skill)
    }

    /// Get the effective system prompt with crew context
    pub fn effective_system_prompt(&self) -> String {
        format!(
            "You are {}, {}.\n\n{}",
            self.name, self.description, self.system_prompt
        )
    }
}

/// Built-in crew templates
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CrewTemplate {
    CodeReviewer,
    BugHunter,
    DocWriter,
    Refactorer,
    TestEngineer,
    SecurityAuditor,
    DevOpsEngineer,
}

impl CrewTemplate {
    /// Create a crew from template
    pub fn create(&self) -> Crew {
        match self {
            CrewTemplate::CodeReviewer => Crew {
                id: "code-reviewer".to_string(),
                name: "Code Reviewer".to_string(),
                description: "Expert code reviewer focused on quality and best practices".to_string(),
                system_prompt: r#"You are an expert code reviewer. Your responsibilities:

1. Review code for bugs, security issues, and performance problems
2. Suggest improvements following best practices
3. Check for proper error handling and edge cases
4. Ensure code is readable and maintainable
5. Verify naming conventions and code style

Be thorough but constructive. Explain the 'why' behind suggestions.
Prioritize issues by severity: Critical > High > Medium > Low."#.to_string(),
                config: CrewConfig {
                    temperature: Some(0.3),
                    ..Default::default()
                },
                permissions: CrewPermissions {
                    file_write: false, // Read-only for safety
                    shell_access: false,
                    ..Default::default()
                },
                tags: vec!["review".to_string(), "quality".to_string()],
                author: Some("Webrana Team".to_string()),
                version: "1.0.0".to_string(),
                created_at: Some(chrono_lite()),
            },
            
            CrewTemplate::BugHunter => Crew {
                id: "bug-hunter".to_string(),
                name: "Bug Hunter".to_string(),
                description: "Specialized in finding and fixing bugs".to_string(),
                system_prompt: r#"You are a bug hunting specialist. Your mission:

1. Analyze code to find potential bugs and issues
2. Identify edge cases that might cause failures
3. Look for race conditions, memory leaks, and resource issues
4. Check error handling paths
5. Propose fixes with clear explanations

Use systematic debugging approaches. Always verify fixes don't introduce new issues."#.to_string(),
                config: CrewConfig {
                    temperature: Some(0.2),
                    auto_mode: true,
                    ..Default::default()
                },
                permissions: CrewPermissions::default(),
                tags: vec!["debug".to_string(), "bugs".to_string()],
                author: Some("Webrana Team".to_string()),
                version: "1.0.0".to_string(),
                created_at: Some(chrono_lite()),
            },

            CrewTemplate::DocWriter => Crew {
                id: "doc-writer".to_string(),
                name: "Documentation Writer".to_string(),
                description: "Creates clear and comprehensive documentation".to_string(),
                system_prompt: r#"You are a technical documentation specialist. Your focus:

1. Write clear, concise documentation
2. Create README files, API docs, and guides
3. Add inline code comments where helpful
4. Generate examples and usage patterns
5. Keep docs up-to-date with code changes

Use markdown formatting. Include code examples. Write for your audience level."#.to_string(),
                config: CrewConfig {
                    temperature: Some(0.5),
                    ..Default::default()
                },
                permissions: CrewPermissions {
                    shell_access: false,
                    ..Default::default()
                },
                tags: vec!["docs".to_string(), "writing".to_string()],
                author: Some("Webrana Team".to_string()),
                version: "1.0.0".to_string(),
                created_at: Some(chrono_lite()),
            },

            CrewTemplate::Refactorer => Crew {
                id: "refactorer".to_string(),
                name: "Code Refactorer".to_string(),
                description: "Improves code structure without changing behavior".to_string(),
                system_prompt: r#"You are a code refactoring expert. Your principles:

1. Improve code structure while preserving behavior
2. Apply SOLID principles and design patterns
3. Reduce complexity and technical debt
4. Improve naming and organization
5. Extract reusable components

Always ensure tests pass after refactoring. Make small, incremental changes."#.to_string(),
                config: CrewConfig {
                    temperature: Some(0.3),
                    auto_mode: true,
                    max_iterations: 15,
                    ..Default::default()
                },
                permissions: CrewPermissions::default(),
                tags: vec!["refactor".to_string(), "clean-code".to_string()],
                author: Some("Webrana Team".to_string()),
                version: "1.0.0".to_string(),
                created_at: Some(chrono_lite()),
            },

            CrewTemplate::TestEngineer => Crew {
                id: "test-engineer".to_string(),
                name: "Test Engineer".to_string(),
                description: "Creates comprehensive test suites".to_string(),
                system_prompt: r#"You are a test engineering specialist. Your responsibilities:

1. Write unit tests with good coverage
2. Create integration and e2e tests
3. Identify edge cases and boundary conditions
4. Set up test fixtures and mocks
5. Ensure tests are fast and reliable

Follow testing best practices. Use appropriate assertions. Test behavior, not implementation."#.to_string(),
                config: CrewConfig {
                    temperature: Some(0.3),
                    auto_mode: true,
                    ..Default::default()
                },
                permissions: CrewPermissions::default(),
                tags: vec!["testing".to_string(), "quality".to_string()],
                author: Some("Webrana Team".to_string()),
                version: "1.0.0".to_string(),
                created_at: Some(chrono_lite()),
            },

            CrewTemplate::SecurityAuditor => Crew {
                id: "security-auditor".to_string(),
                name: "Security Auditor".to_string(),
                description: "Identifies security vulnerabilities".to_string(),
                system_prompt: r#"You are a security auditing specialist. Your focus:

1. Identify security vulnerabilities (OWASP Top 10)
2. Check for injection attacks, XSS, CSRF
3. Review authentication and authorization
4. Find hardcoded secrets and credentials
5. Assess dependency vulnerabilities

Report findings with severity levels. Provide remediation guidance."#.to_string(),
                config: CrewConfig {
                    temperature: Some(0.2),
                    ..Default::default()
                },
                permissions: CrewPermissions {
                    file_write: false,
                    shell_access: false,
                    ..Default::default()
                },
                tags: vec!["security".to_string(), "audit".to_string()],
                author: Some("Webrana Team".to_string()),
                version: "1.0.0".to_string(),
                created_at: Some(chrono_lite()),
            },

            CrewTemplate::DevOpsEngineer => Crew {
                id: "devops-engineer".to_string(),
                name: "DevOps Engineer".to_string(),
                description: "Handles CI/CD, infrastructure, and deployment".to_string(),
                system_prompt: r#"You are a DevOps engineering specialist. Your expertise:

1. Set up CI/CD pipelines (GitHub Actions, GitLab CI)
2. Write Dockerfiles and docker-compose configs
3. Configure Kubernetes deployments
4. Set up monitoring and logging
5. Automate infrastructure tasks

Follow infrastructure-as-code principles. Prioritize security and reliability."#.to_string(),
                config: CrewConfig {
                    temperature: Some(0.3),
                    auto_mode: true,
                    ..Default::default()
                },
                permissions: CrewPermissions::default(),
                tags: vec!["devops".to_string(), "infrastructure".to_string()],
                author: Some("Webrana Team".to_string()),
                version: "1.0.0".to_string(),
                created_at: Some(chrono_lite()),
            },
        }
    }

    /// List all available templates
    pub fn all() -> Vec<CrewTemplate> {
        vec![
            CrewTemplate::CodeReviewer,
            CrewTemplate::BugHunter,
            CrewTemplate::DocWriter,
            CrewTemplate::Refactorer,
            CrewTemplate::TestEngineer,
            CrewTemplate::SecurityAuditor,
            CrewTemplate::DevOpsEngineer,
        ]
    }

    /// Get template by name
    pub fn from_name(name: &str) -> Option<CrewTemplate> {
        match name.to_lowercase().as_str() {
            "code-reviewer" | "reviewer" => Some(CrewTemplate::CodeReviewer),
            "bug-hunter" | "debugger" => Some(CrewTemplate::BugHunter),
            "doc-writer" | "docs" => Some(CrewTemplate::DocWriter),
            "refactorer" | "refactor" => Some(CrewTemplate::Refactorer),
            "test-engineer" | "tester" => Some(CrewTemplate::TestEngineer),
            "security-auditor" | "security" => Some(CrewTemplate::SecurityAuditor),
            "devops-engineer" | "devops" => Some(CrewTemplate::DevOpsEngineer),
            _ => None,
        }
    }
}

fn chrono_lite() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    format!("{}", secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crew_creation() {
        let crew = Crew::new(
            "my-crew",
            "My Crew",
            "A helpful assistant",
            "You are helpful."
        );
        assert_eq!(crew.id, "my-crew");
        assert_eq!(crew.name, "My Crew");
    }

    #[test]
    fn test_skill_permissions() {
        let mut crew = Crew::new("test", "Test", "Test", "Test");
        
        // All allowed by default
        assert!(crew.is_skill_allowed("read_file"));
        
        // Add to denied
        crew.permissions.denied_skills.insert("shell_execute".to_string());
        assert!(!crew.is_skill_allowed("shell_execute"));
        
        // Allowed list
        crew.permissions.allowed_skills.insert("read_file".to_string());
        assert!(crew.is_skill_allowed("read_file"));
        assert!(!crew.is_skill_allowed("write_file")); // Not in allowed list
    }

    #[test]
    fn test_templates() {
        let templates = CrewTemplate::all();
        assert_eq!(templates.len(), 7);
        
        let reviewer = CrewTemplate::CodeReviewer.create();
        assert_eq!(reviewer.id, "code-reviewer");
        assert!(!reviewer.permissions.file_write); // Read-only
    }

    #[test]
    fn test_template_lookup() {
        assert_eq!(CrewTemplate::from_name("reviewer"), Some(CrewTemplate::CodeReviewer));
        assert_eq!(CrewTemplate::from_name("devops"), Some(CrewTemplate::DevOpsEngineer));
        assert_eq!(CrewTemplate::from_name("unknown"), None);
    }
}
