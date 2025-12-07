use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;

use super::index::{FileEntry, FileType};

pub struct FileWalker {
    root: PathBuf,
    ignore_patterns: Vec<String>,
    default_ignores: HashSet<String>,
}

impl FileWalker {
    pub fn new(root: impl AsRef<Path>) -> Self {
        let mut default_ignores = HashSet::new();
        default_ignores.insert(".git".to_string());
        default_ignores.insert("node_modules".to_string());
        default_ignores.insert("target".to_string());
        default_ignores.insert(".venv".to_string());
        default_ignores.insert("venv".to_string());
        default_ignores.insert("__pycache__".to_string());
        default_ignores.insert(".cache".to_string());
        default_ignores.insert("dist".to_string());
        default_ignores.insert("build".to_string());
        default_ignores.insert(".next".to_string());
        default_ignores.insert(".nuxt".to_string());

        Self {
            root: root.as_ref().to_path_buf(),
            ignore_patterns: Vec::new(),
            default_ignores,
        }
    }

    pub fn load_gitignore(&mut self) -> Result<()> {
        let gitignore_path = self.root.join(".gitignore");
        if gitignore_path.exists() {
            let content = fs::read_to_string(&gitignore_path)?;
            for line in content.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with('#') {
                    self.ignore_patterns.push(line.to_string());
                }
            }
        }
        Ok(())
    }

    pub fn walk(&self) -> Result<Vec<FileEntry>> {
        let mut entries = Vec::new();
        self.walk_dir(&self.root, &mut entries, 0)?;
        Ok(entries)
    }

    fn walk_dir(&self, dir: &Path, entries: &mut Vec<FileEntry>, depth: usize) -> Result<()> {
        if depth > 20 {
            return Ok(());
        }

        let read_dir = match fs::read_dir(dir) {
            Ok(rd) => rd,
            Err(_) => return Ok(()),
        };

        for entry in read_dir.flatten() {
            let path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();

            if self.should_ignore(&file_name, &path) {
                continue;
            }

            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            let relative_path = path.strip_prefix(&self.root)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();

            if metadata.is_dir() {
                entries.push(FileEntry {
                    path: relative_path.clone(),
                    file_type: FileType::Directory,
                    size: 0,
                    extension: None,
                });
                self.walk_dir(&path, entries, depth + 1)?;
            } else if metadata.is_file() {
                let extension = path.extension()
                    .map(|e| e.to_string_lossy().to_string());
                let file_type = Self::detect_file_type(&extension);

                entries.push(FileEntry {
                    path: relative_path,
                    file_type,
                    size: metadata.len(),
                    extension,
                });
            }
        }

        Ok(())
    }

    fn should_ignore(&self, name: &str, path: &Path) -> bool {
        if name.starts_with('.') && name != ".env.example" {
            return true;
        }

        if self.default_ignores.contains(name) {
            return true;
        }

        let relative = path.strip_prefix(&self.root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        for pattern in &self.ignore_patterns {
            if self.matches_pattern(&relative, pattern) || self.matches_pattern(name, pattern) {
                return true;
            }
        }

        false
    }

    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        let pattern = pattern.trim_start_matches('/');
        let pattern = pattern.trim_end_matches('/');

        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let (prefix, suffix) = (parts[0], parts[1]);
                return path.starts_with(prefix) && path.ends_with(suffix);
            }
        }

        path == pattern || path.starts_with(&format!("{}/", pattern)) || path.ends_with(&format!("/{}", pattern))
    }

    fn detect_file_type(extension: &Option<String>) -> FileType {
        match extension.as_deref() {
            Some("rs") | Some("py") | Some("js") | Some("ts") | Some("go") |
            Some("java") | Some("c") | Some("cpp") | Some("h") | Some("hpp") |
            Some("rb") | Some("php") | Some("swift") | Some("kt") | Some("scala") |
            Some("jsx") | Some("tsx") | Some("vue") | Some("svelte") => FileType::Code,

            Some("md") | Some("txt") | Some("rst") | Some("adoc") => FileType::Document,

            Some("json") | Some("yaml") | Some("yml") | Some("toml") | Some("xml") |
            Some("ini") | Some("conf") | Some("cfg") => FileType::Config,

            Some("sh") | Some("bash") | Some("zsh") | Some("fish") | Some("ps1") |
            Some("bat") | Some("cmd") => FileType::Script,

            Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("svg") |
            Some("ico") | Some("webp") => FileType::Image,

            Some("css") | Some("scss") | Some("sass") | Some("less") => FileType::Style,

            Some("html") | Some("htm") => FileType::Markup,

            Some("sql") => FileType::Database,

            Some("lock") => FileType::Lock,

            _ => FileType::Other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_matching() {
        let walker = FileWalker::new("/tmp");
        assert!(walker.matches_pattern("node_modules/test", "node_modules"));
        assert!(walker.matches_pattern("test.log", "*.log"));
    }
}
