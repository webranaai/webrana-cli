use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[allow(unused_imports)]
use crate::indexer::{FileIndex, FileType, FileWalker, ProjectDetector, ProjectInfo};

#[derive(Debug, Serialize, Deserialize)]
pub struct CodebaseContext {
    pub project_info: ProjectInfo,
    pub file_summary: String,
    pub file_tree: String,
    pub code_files: Vec<String>,
    pub recent_files: Vec<String>,
}

pub struct CodebaseSkill {
    root: std::path::PathBuf,
    index: Option<FileIndex>,
    project_info: Option<ProjectInfo>,
}

impl CodebaseSkill {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            index: None,
            project_info: None,
        }
    }

    pub fn index(&mut self) -> Result<&FileIndex> {
        if self.index.is_none() {
            let mut walker = FileWalker::new(&self.root);
            walker.load_gitignore()?;
            let entries = walker.walk()?;
            self.index = Some(FileIndex::build(entries));
        }
        Ok(self.index.as_ref().unwrap())
    }

    pub fn detect_project(&mut self) -> Result<&ProjectInfo> {
        if self.project_info.is_none() {
            let detector = ProjectDetector::new(&self.root);
            self.project_info = Some(detector.detect()?);
        }
        Ok(self.project_info.as_ref().unwrap())
    }

    pub fn get_context(&mut self, max_files: usize) -> Result<CodebaseContext> {
        let project_info = self.detect_project()?.clone();
        let index = self.index()?;

        let code_files: Vec<String> = index
            .get_code_files()
            .iter()
            .take(max_files)
            .map(|f| f.path.clone())
            .collect();

        Ok(CodebaseContext {
            project_info,
            file_summary: index.summary(),
            file_tree: index.tree(3),
            code_files,
            recent_files: Vec::new(),
        })
    }

    pub fn search_files(&mut self, query: &str) -> Result<Vec<String>> {
        let index = self.index()?;
        let results = index.search(query);
        Ok(results.iter().map(|f| f.path.clone()).collect())
    }

    pub fn get_file_content(&self, path: &str) -> Result<String> {
        let full_path = self.root.join(path);
        Ok(fs::read_to_string(full_path)?)
    }

    pub fn grep(&self, pattern: &str) -> Result<Vec<GrepResult>> {
        let mut results = Vec::new();
        self.grep_recursive(&self.root, pattern, &mut results, 0)?;
        Ok(results)
    }

    fn grep_recursive(
        &self,
        dir: &Path,
        pattern: &str,
        results: &mut Vec<GrepResult>,
        depth: usize,
    ) -> Result<()> {
        if depth > 10 || results.len() > 100 {
            return Ok(());
        }

        let default_ignores = vec![".git", "node_modules", "target", ".venv", "__pycache__"];

        for entry in fs::read_dir(dir)?.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if default_ignores.contains(&name.as_str()) || name.starts_with('.') {
                continue;
            }

            if path.is_dir() {
                self.grep_recursive(&path, pattern, results, depth + 1)?;
            } else if path.is_file() {
                if let Ok(content) = fs::read_to_string(&path) {
                    let relative_path = path
                        .strip_prefix(&self.root)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .to_string();

                    for (line_num, line) in content.lines().enumerate() {
                        if line.to_lowercase().contains(&pattern.to_lowercase()) {
                            results.push(GrepResult {
                                file: relative_path.clone(),
                                line_number: line_num + 1,
                                content: line.to_string(),
                            });
                            if results.len() >= 100 {
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn list_symbols(&self, path: &str) -> Result<Vec<Symbol>> {
        let full_path = self.root.join(path);
        let content = fs::read_to_string(&full_path)?;
        let extension = Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let mut symbols = Vec::new();

        match extension {
            "rs" => self.extract_rust_symbols(&content, &mut symbols),
            "py" => self.extract_python_symbols(&content, &mut symbols),
            "js" | "ts" | "jsx" | "tsx" => self.extract_js_symbols(&content, &mut symbols),
            "go" => self.extract_go_symbols(&content, &mut symbols),
            _ => {}
        }

        Ok(symbols)
    }

    fn extract_rust_symbols(&self, content: &str, symbols: &mut Vec<Symbol>) {
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") {
                if let Some(name) = self.extract_fn_name(trimmed, "fn ") {
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Function,
                        line: line_num + 1,
                    });
                }
            } else if trimmed.starts_with("struct ") || trimmed.starts_with("pub struct ") {
                if let Some(name) = self.extract_after_keyword(trimmed, "struct ") {
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Struct,
                        line: line_num + 1,
                    });
                }
            } else if trimmed.starts_with("enum ") || trimmed.starts_with("pub enum ") {
                if let Some(name) = self.extract_after_keyword(trimmed, "enum ") {
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Enum,
                        line: line_num + 1,
                    });
                }
            } else if trimmed.starts_with("trait ") || trimmed.starts_with("pub trait ") {
                if let Some(name) = self.extract_after_keyword(trimmed, "trait ") {
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Trait,
                        line: line_num + 1,
                    });
                }
            } else if trimmed.starts_with("impl ") {
                if let Some(name) = self.extract_impl_name(trimmed) {
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Impl,
                        line: line_num + 1,
                    });
                }
            }
        }
    }

    fn extract_python_symbols(&self, content: &str, symbols: &mut Vec<Symbol>) {
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("def ") {
                if let Some(name) = self.extract_fn_name(trimmed, "def ") {
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Function,
                        line: line_num + 1,
                    });
                }
            } else if trimmed.starts_with("class ") {
                if let Some(name) = self.extract_class_name(trimmed) {
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Class,
                        line: line_num + 1,
                    });
                }
            } else if trimmed.starts_with("async def ") {
                if let Some(name) = self.extract_fn_name(trimmed, "async def ") {
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Function,
                        line: line_num + 1,
                    });
                }
            }
        }
    }

    fn extract_js_symbols(&self, content: &str, symbols: &mut Vec<Symbol>) {
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("function ") {
                if let Some(name) = self.extract_fn_name(trimmed, "function ") {
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Function,
                        line: line_num + 1,
                    });
                }
            } else if trimmed.starts_with("class ") {
                if let Some(name) = self.extract_class_name(trimmed) {
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Class,
                        line: line_num + 1,
                    });
                }
            } else if trimmed.contains("const ")
                && trimmed.contains(" = ")
                && (trimmed.contains("=>") || trimmed.contains("function"))
            {
                if let Some(name) = self.extract_const_fn(trimmed) {
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Function,
                        line: line_num + 1,
                    });
                }
            } else if trimmed.starts_with("export ") {
                if trimmed.contains("function ") {
                    if let Some(name) = self.extract_fn_name(trimmed, "function ") {
                        symbols.push(Symbol {
                            name,
                            kind: SymbolKind::Function,
                            line: line_num + 1,
                        });
                    }
                }
            }
        }
    }

    fn extract_go_symbols(&self, content: &str, symbols: &mut Vec<Symbol>) {
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("func ") {
                if let Some(name) = self.extract_go_func_name(trimmed) {
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Function,
                        line: line_num + 1,
                    });
                }
            } else if trimmed.starts_with("type ") && trimmed.contains(" struct") {
                if let Some(name) = self.extract_after_keyword(trimmed, "type ") {
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Struct,
                        line: line_num + 1,
                    });
                }
            } else if trimmed.starts_with("type ") && trimmed.contains(" interface") {
                if let Some(name) = self.extract_after_keyword(trimmed, "type ") {
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Interface,
                        line: line_num + 1,
                    });
                }
            }
        }
    }

    fn extract_fn_name(&self, line: &str, keyword: &str) -> Option<String> {
        let after_keyword = line.split(keyword).nth(1)?;
        let name = after_keyword.split('(').next()?.trim();
        if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        }
    }

    fn extract_after_keyword(&self, line: &str, keyword: &str) -> Option<String> {
        let after_keyword = line.split(keyword).nth(1)?;
        let name = after_keyword
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .next()?
            .trim();
        if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        }
    }

    fn extract_class_name(&self, line: &str) -> Option<String> {
        let after_class = line.split("class ").nth(1)?;
        let name = after_class
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .next()?
            .trim();
        if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        }
    }

    fn extract_impl_name(&self, line: &str) -> Option<String> {
        let after_impl = line.split("impl ").nth(1)?;
        let cleaned = after_impl.split('{').next()?.trim();
        if cleaned.contains(" for ") {
            let parts: Vec<&str> = cleaned.split(" for ").collect();
            if parts.len() == 2 {
                return Some(format!("{} for {}", parts[0].trim(), parts[1].trim()));
            }
        }
        let name = cleaned
            .split(|c: char| c == '<' || c == '{' || c.is_whitespace())
            .next()?
            .trim();
        if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        }
    }

    fn extract_const_fn(&self, line: &str) -> Option<String> {
        let after_const = line.split("const ").nth(1)?;
        let name = after_const
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .next()?
            .trim();
        if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        }
    }

    fn extract_go_func_name(&self, line: &str) -> Option<String> {
        let after_func = line.split("func ").nth(1)?;
        if after_func.starts_with('(') {
            let after_receiver = after_func.split(')').nth(1)?;
            let name = after_receiver.trim().split('(').next()?.trim();
            if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            }
        } else {
            let name = after_func.split('(').next()?.trim();
            if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrepResult {
    pub file: String,
    pub line_number: usize,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SymbolKind {
    Function,
    Class,
    Struct,
    Enum,
    Trait,
    Interface,
    Impl,
    Variable,
    Constant,
}

impl SymbolKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SymbolKind::Function => "function",
            SymbolKind::Class => "class",
            SymbolKind::Struct => "struct",
            SymbolKind::Enum => "enum",
            SymbolKind::Trait => "trait",
            SymbolKind::Interface => "interface",
            SymbolKind::Impl => "impl",
            SymbolKind::Variable => "variable",
            SymbolKind::Constant => "constant",
        }
    }
}
