use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileType {
    Code,
    Document,
    Config,
    Script,
    Image,
    Style,
    Markup,
    Database,
    Lock,
    Directory,
    Other,
}

impl FileType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FileType::Code => "code",
            FileType::Document => "document",
            FileType::Config => "config",
            FileType::Script => "script",
            FileType::Image => "image",
            FileType::Style => "style",
            FileType::Markup => "markup",
            FileType::Database => "database",
            FileType::Lock => "lock",
            FileType::Directory => "directory",
            FileType::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub file_type: FileType,
    pub size: u64,
    pub extension: Option<String>,
}

#[derive(Debug, Default)]
pub struct FileIndex {
    pub entries: Vec<FileEntry>,
    pub by_extension: HashMap<String, Vec<usize>>,
    pub by_type: HashMap<String, Vec<usize>>,
    pub total_size: u64,
    pub code_files: usize,
    pub config_files: usize,
}

impl FileIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(entries: Vec<FileEntry>) -> Self {
        let mut index = Self::new();
        index.total_size = entries.iter().map(|e| e.size).sum();
        index.code_files = entries
            .iter()
            .filter(|e| e.file_type == FileType::Code)
            .count();
        index.config_files = entries
            .iter()
            .filter(|e| e.file_type == FileType::Config)
            .count();

        for (i, entry) in entries.iter().enumerate() {
            if let Some(ext) = &entry.extension {
                index.by_extension.entry(ext.clone()).or_default().push(i);
            }

            index
                .by_type
                .entry(entry.file_type.as_str().to_string())
                .or_default()
                .push(i);
        }

        index.entries = entries;
        index
    }

    pub fn get_code_files(&self) -> Vec<&FileEntry> {
        self.entries
            .iter()
            .filter(|e| e.file_type == FileType::Code)
            .collect()
    }

    pub fn get_by_extension(&self, ext: &str) -> Vec<&FileEntry> {
        self.by_extension
            .get(ext)
            .map(|indices| indices.iter().map(|&i| &self.entries[i]).collect())
            .unwrap_or_default()
    }

    pub fn search(&self, query: &str) -> Vec<&FileEntry> {
        let query_lower = query.to_lowercase();
        self.entries
            .iter()
            .filter(|e| e.path.to_lowercase().contains(&query_lower))
            .collect()
    }

    pub fn summary(&self) -> String {
        let dirs = self
            .entries
            .iter()
            .filter(|e| e.file_type == FileType::Directory)
            .count();
        let files = self.entries.len() - dirs;

        format!(
            "{} files, {} directories, {} code files, {:.2} KB total",
            files,
            dirs,
            self.code_files,
            self.total_size as f64 / 1024.0
        )
    }

    pub fn tree(&self, max_depth: usize) -> String {
        let mut output = String::new();
        let mut current_depth = 0;

        for entry in &self.entries {
            let depth = entry.path.matches('/').count();
            if depth > max_depth {
                continue;
            }

            let indent = "  ".repeat(depth);
            let name = entry.path.split('/').last().unwrap_or(&entry.path);

            let icon = match entry.file_type {
                FileType::Directory => "📁",
                FileType::Code => "📄",
                FileType::Config => "⚙️",
                FileType::Document => "📝",
                FileType::Script => "📜",
                _ => "📎",
            };

            output.push_str(&format!("{}{} {}\n", indent, icon, name));
        }

        output
    }
}
