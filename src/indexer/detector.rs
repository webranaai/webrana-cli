use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProjectType {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Java,
    Ruby,
    Php,
    CSharp,
    Cpp,
    Swift,
    Kotlin,
    Unknown,
}

impl ProjectType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProjectType::Rust => "rust",
            ProjectType::Python => "python",
            ProjectType::JavaScript => "javascript",
            ProjectType::TypeScript => "typescript",
            ProjectType::Go => "go",
            ProjectType::Java => "java",
            ProjectType::Ruby => "ruby",
            ProjectType::Php => "php",
            ProjectType::CSharp => "csharp",
            ProjectType::Cpp => "cpp",
            ProjectType::Swift => "swift",
            ProjectType::Kotlin => "kotlin",
            ProjectType::Unknown => "unknown",
        }
    }

    pub fn file_extensions(&self) -> Vec<&'static str> {
        match self {
            ProjectType::Rust => vec!["rs"],
            ProjectType::Python => vec!["py"],
            ProjectType::JavaScript => vec!["js", "jsx", "mjs"],
            ProjectType::TypeScript => vec!["ts", "tsx"],
            ProjectType::Go => vec!["go"],
            ProjectType::Java => vec!["java"],
            ProjectType::Ruby => vec!["rb"],
            ProjectType::Php => vec!["php"],
            ProjectType::CSharp => vec!["cs"],
            ProjectType::Cpp => vec!["cpp", "cc", "cxx", "c", "h", "hpp"],
            ProjectType::Swift => vec!["swift"],
            ProjectType::Kotlin => vec!["kt", "kts"],
            ProjectType::Unknown => vec![],
        }
    }

    pub fn build_command(&self) -> Option<&'static str> {
        match self {
            ProjectType::Rust => Some("cargo build"),
            ProjectType::Python => None,
            ProjectType::JavaScript => Some("npm run build"),
            ProjectType::TypeScript => Some("npm run build"),
            ProjectType::Go => Some("go build"),
            ProjectType::Java => Some("mvn compile"),
            ProjectType::Ruby => None,
            ProjectType::Php => None,
            ProjectType::CSharp => Some("dotnet build"),
            ProjectType::Cpp => Some("make"),
            ProjectType::Swift => Some("swift build"),
            ProjectType::Kotlin => Some("gradle build"),
            ProjectType::Unknown => None,
        }
    }

    pub fn test_command(&self) -> Option<&'static str> {
        match self {
            ProjectType::Rust => Some("cargo test"),
            ProjectType::Python => Some("pytest"),
            ProjectType::JavaScript => Some("npm test"),
            ProjectType::TypeScript => Some("npm test"),
            ProjectType::Go => Some("go test ./..."),
            ProjectType::Java => Some("mvn test"),
            ProjectType::Ruby => Some("bundle exec rspec"),
            ProjectType::Php => Some("phpunit"),
            ProjectType::CSharp => Some("dotnet test"),
            ProjectType::Cpp => Some("make test"),
            ProjectType::Swift => Some("swift test"),
            ProjectType::Kotlin => Some("gradle test"),
            ProjectType::Unknown => None,
        }
    }

    pub fn lint_command(&self) -> Option<&'static str> {
        match self {
            ProjectType::Rust => Some("cargo clippy"),
            ProjectType::Python => Some("ruff check ."),
            ProjectType::JavaScript => Some("eslint ."),
            ProjectType::TypeScript => Some("eslint ."),
            ProjectType::Go => Some("golangci-lint run"),
            ProjectType::Java => Some("mvn checkstyle:check"),
            ProjectType::Ruby => Some("rubocop"),
            ProjectType::Php => Some("phpcs"),
            ProjectType::CSharp => Some("dotnet format --verify-no-changes"),
            ProjectType::Cpp => Some("clang-tidy"),
            ProjectType::Swift => Some("swiftlint"),
            ProjectType::Kotlin => Some("ktlint"),
            ProjectType::Unknown => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub project_type: ProjectType,
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub dependencies: Vec<String>,
    pub config_file: Option<String>,
    pub has_git: bool,
    pub has_tests: bool,
    pub has_ci: bool,
}

impl Default for ProjectInfo {
    fn default() -> Self {
        Self {
            project_type: ProjectType::Unknown,
            name: None,
            version: None,
            description: None,
            dependencies: Vec::new(),
            config_file: None,
            has_git: false,
            has_tests: false,
            has_ci: false,
        }
    }
}

pub struct ProjectDetector {
    root: std::path::PathBuf,
}

impl ProjectDetector {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    pub fn detect(&self) -> Result<ProjectInfo> {
        let mut info = ProjectInfo::default();

        info.has_git = self.root.join(".git").exists();
        info.has_ci = self.root.join(".github/workflows").exists() 
            || self.root.join(".gitlab-ci.yml").exists()
            || self.root.join(".circleci").exists();

        if let Some((pt, config)) = self.detect_project_type() {
            info.project_type = pt;
            info.config_file = Some(config);
        }

        match info.project_type {
            ProjectType::Rust => self.parse_cargo_toml(&mut info),
            ProjectType::JavaScript | ProjectType::TypeScript => self.parse_package_json(&mut info),
            ProjectType::Python => self.parse_pyproject(&mut info),
            ProjectType::Go => self.parse_go_mod(&mut info),
            _ => {}
        }

        info.has_tests = self.detect_tests(&info.project_type);

        Ok(info)
    }

    fn detect_project_type(&self) -> Option<(ProjectType, String)> {
        let markers = [
            ("Cargo.toml", ProjectType::Rust),
            ("package.json", ProjectType::JavaScript),
            ("tsconfig.json", ProjectType::TypeScript),
            ("pyproject.toml", ProjectType::Python),
            ("setup.py", ProjectType::Python),
            ("requirements.txt", ProjectType::Python),
            ("go.mod", ProjectType::Go),
            ("pom.xml", ProjectType::Java),
            ("build.gradle", ProjectType::Java),
            ("Gemfile", ProjectType::Ruby),
            ("composer.json", ProjectType::Php),
            ("Package.swift", ProjectType::Swift),
            ("CMakeLists.txt", ProjectType::Cpp),
            ("Makefile", ProjectType::Cpp),
        ];

        for (file, project_type) in markers {
            if self.root.join(file).exists() {
                if file == "package.json" && self.root.join("tsconfig.json").exists() {
                    return Some((ProjectType::TypeScript, "tsconfig.json".to_string()));
                }
                return Some((project_type, file.to_string()));
            }
        }

        None
    }

    fn parse_cargo_toml(&self, info: &mut ProjectInfo) {
        let path = self.root.join("Cargo.toml");
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(parsed) = content.parse::<toml::Table>() {
                if let Some(package) = parsed.get("package").and_then(|p| p.as_table()) {
                    info.name = package.get("name").and_then(|n| n.as_str()).map(String::from);
                    info.version = package.get("version").and_then(|v| v.as_str()).map(String::from);
                    info.description = package.get("description").and_then(|d| d.as_str()).map(String::from);
                }
                if let Some(deps) = parsed.get("dependencies").and_then(|d| d.as_table()) {
                    info.dependencies = deps.keys().cloned().collect();
                }
            }
        }
    }

    fn parse_package_json(&self, info: &mut ProjectInfo) {
        let path = self.root.join("package.json");
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                info.name = parsed.get("name").and_then(|n| n.as_str()).map(String::from);
                info.version = parsed.get("version").and_then(|v| v.as_str()).map(String::from);
                info.description = parsed.get("description").and_then(|d| d.as_str()).map(String::from);
                
                if let Some(deps) = parsed.get("dependencies").and_then(|d| d.as_object()) {
                    info.dependencies.extend(deps.keys().cloned());
                }
                if let Some(deps) = parsed.get("devDependencies").and_then(|d| d.as_object()) {
                    info.dependencies.extend(deps.keys().cloned());
                }
            }
        }
    }

    fn parse_pyproject(&self, info: &mut ProjectInfo) {
        let path = self.root.join("pyproject.toml");
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(parsed) = content.parse::<toml::Table>() {
                if let Some(project) = parsed.get("project").and_then(|p| p.as_table()) {
                    info.name = project.get("name").and_then(|n| n.as_str()).map(String::from);
                    info.version = project.get("version").and_then(|v| v.as_str()).map(String::from);
                    info.description = project.get("description").and_then(|d| d.as_str()).map(String::from);
                }
            }
        }
    }

    fn parse_go_mod(&self, info: &mut ProjectInfo) {
        let path = self.root.join("go.mod");
        if let Ok(content) = fs::read_to_string(&path) {
            for line in content.lines() {
                if line.starts_with("module ") {
                    info.name = Some(line.trim_start_matches("module ").trim().to_string());
                    break;
                }
            }
        }
    }

    fn detect_tests(&self, project_type: &ProjectType) -> bool {
        let test_dirs = ["tests", "test", "spec", "__tests__"];
        for dir in test_dirs {
            if self.root.join(dir).exists() {
                return true;
            }
        }

        match project_type {
            ProjectType::Rust => {
                if let Ok(content) = fs::read_to_string(self.root.join("src/main.rs")) {
                    return content.contains("#[cfg(test)]") || content.contains("#[test]");
                }
            }
            ProjectType::Python => {
                return self.root.join("pytest.ini").exists() 
                    || self.root.join("setup.cfg").exists();
            }
            _ => {}
        }

        false
    }
}

impl std::fmt::Display for ProjectInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Project Type: {}", self.project_type.as_str())?;
        if let Some(name) = &self.name {
            writeln!(f, "Name: {}", name)?;
        }
        if let Some(version) = &self.version {
            writeln!(f, "Version: {}", version)?;
        }
        if let Some(desc) = &self.description {
            writeln!(f, "Description: {}", desc)?;
        }
        writeln!(f, "Git: {}", if self.has_git { "yes" } else { "no" })?;
        writeln!(f, "Tests: {}", if self.has_tests { "yes" } else { "no" })?;
        writeln!(f, "CI: {}", if self.has_ci { "yes" } else { "no" })?;
        if !self.dependencies.is_empty() {
            writeln!(f, "Dependencies: {} packages", self.dependencies.len())?;
        }
        Ok(())
    }
}
