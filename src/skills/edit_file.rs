use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditOperation {
    pub search: String,
    pub replace: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditResult {
    pub success: bool,
    pub file_path: String,
    pub changes_made: usize,
    pub message: String,
}

pub struct EditFileSkill;

impl EditFileSkill {
    pub fn new() -> Self {
        Self
    }

    pub fn edit_file(&self, path: &str, search: &str, replace: &str) -> Result<EditResult> {
        let file_path = Path::new(path);

        if !file_path.exists() {
            return Ok(EditResult {
                success: false,
                file_path: path.to_string(),
                changes_made: 0,
                message: format!("File not found: {}", path),
            });
        }

        let content = fs::read_to_string(file_path)?;

        if !content.contains(search) {
            return Ok(EditResult {
                success: false,
                file_path: path.to_string(),
                changes_made: 0,
                message: "Search string not found in file".to_string(),
            });
        }

        let changes = content.matches(search).count();
        let new_content = content.replace(search, replace);

        fs::write(file_path, &new_content)?;

        Ok(EditResult {
            success: true,
            file_path: path.to_string(),
            changes_made: changes,
            message: format!("Successfully replaced {} occurrence(s)", changes),
        })
    }

    pub fn edit_file_once(&self, path: &str, search: &str, replace: &str) -> Result<EditResult> {
        let file_path = Path::new(path);

        if !file_path.exists() {
            return Ok(EditResult {
                success: false,
                file_path: path.to_string(),
                changes_made: 0,
                message: format!("File not found: {}", path),
            });
        }

        let content = fs::read_to_string(file_path)?;

        if !content.contains(search) {
            return Ok(EditResult {
                success: false,
                file_path: path.to_string(),
                changes_made: 0,
                message: "Search string not found in file".to_string(),
            });
        }

        let new_content = content.replacen(search, replace, 1);
        fs::write(file_path, &new_content)?;

        Ok(EditResult {
            success: true,
            file_path: path.to_string(),
            changes_made: 1,
            message: "Successfully replaced first occurrence".to_string(),
        })
    }

    pub fn apply_diff(&self, path: &str, diff_content: &str) -> Result<EditResult> {
        let operations = self.parse_diff(diff_content)?;

        if operations.is_empty() {
            return Ok(EditResult {
                success: false,
                file_path: path.to_string(),
                changes_made: 0,
                message: "No valid edit operations found in diff".to_string(),
            });
        }

        let file_path = Path::new(path);
        let mut content = if file_path.exists() {
            fs::read_to_string(file_path)?
        } else {
            String::new()
        };

        let mut total_changes = 0;
        for op in &operations {
            if content.contains(&op.search) {
                content = content.replace(&op.search, &op.replace);
                total_changes += 1;
            }
        }

        if total_changes == 0 {
            return Ok(EditResult {
                success: false,
                file_path: path.to_string(),
                changes_made: 0,
                message: "No matching search strings found".to_string(),
            });
        }

        fs::write(file_path, &content)?;

        Ok(EditResult {
            success: true,
            file_path: path.to_string(),
            changes_made: total_changes,
            message: format!("Applied {} edit operation(s)", total_changes),
        })
    }

    fn parse_diff(&self, diff: &str) -> Result<Vec<EditOperation>> {
        let mut operations = Vec::new();
        let mut current_search = String::new();
        let mut current_replace = String::new();
        let mut in_search = false;
        let mut in_replace = false;

        for line in diff.lines() {
            if line.starts_with("<<<<<<< SEARCH") {
                in_search = true;
                in_replace = false;
                current_search.clear();
            } else if line.starts_with("=======") {
                in_search = false;
                in_replace = true;
                current_replace.clear();
            } else if line.starts_with(">>>>>>> REPLACE") {
                in_replace = false;
                if !current_search.is_empty() {
                    let search = current_search.trim_end_matches('\n').to_string();
                    let replace = current_replace.trim_end_matches('\n').to_string();
                    operations.push(EditOperation { search, replace });
                }
            } else if in_search {
                current_search.push_str(line);
                current_search.push('\n');
            } else if in_replace {
                current_replace.push_str(line);
                current_replace.push('\n');
            }
        }

        Ok(operations)
    }

    pub fn insert_at_line(
        &self,
        path: &str,
        line_number: usize,
        content: &str,
    ) -> Result<EditResult> {
        let file_path = Path::new(path);

        if !file_path.exists() {
            return Ok(EditResult {
                success: false,
                file_path: path.to_string(),
                changes_made: 0,
                message: format!("File not found: {}", path),
            });
        }

        let file_content = fs::read_to_string(file_path)?;
        let mut lines: Vec<&str> = file_content.lines().collect();

        let insert_at = if line_number == 0 { 0 } else { line_number - 1 };

        if insert_at > lines.len() {
            return Ok(EditResult {
                success: false,
                file_path: path.to_string(),
                changes_made: 0,
                message: format!(
                    "Line number {} exceeds file length {}",
                    line_number,
                    lines.len()
                ),
            });
        }

        lines.insert(insert_at, content);
        let new_content = lines.join("\n");
        fs::write(file_path, &new_content)?;

        Ok(EditResult {
            success: true,
            file_path: path.to_string(),
            changes_made: 1,
            message: format!("Inserted content at line {}", line_number),
        })
    }

    pub fn delete_lines(
        &self,
        path: &str,
        start_line: usize,
        end_line: usize,
    ) -> Result<EditResult> {
        let file_path = Path::new(path);

        if !file_path.exists() {
            return Ok(EditResult {
                success: false,
                file_path: path.to_string(),
                changes_made: 0,
                message: format!("File not found: {}", path),
            });
        }

        let file_content = fs::read_to_string(file_path)?;
        let lines: Vec<&str> = file_content.lines().collect();

        let start = start_line.saturating_sub(1);
        let end = end_line.min(lines.len());

        if start >= lines.len() {
            return Ok(EditResult {
                success: false,
                file_path: path.to_string(),
                changes_made: 0,
                message: "Start line exceeds file length".to_string(),
            });
        }

        let deleted = end - start;
        let new_lines: Vec<&str> = lines
            .iter()
            .enumerate()
            .filter(|(i, _)| *i < start || *i >= end)
            .map(|(_, line)| *line)
            .collect();

        let new_content = new_lines.join("\n");
        fs::write(file_path, &new_content)?;

        Ok(EditResult {
            success: true,
            file_path: path.to_string(),
            changes_made: deleted,
            message: format!("Deleted {} line(s)", deleted),
        })
    }
}

pub struct MultiEditSkill;

impl MultiEditSkill {
    pub fn new() -> Self {
        Self
    }

    pub fn batch_edit(&self, edits: Vec<(String, String, String)>) -> Result<Vec<EditResult>> {
        let skill = EditFileSkill::new();
        let mut results = Vec::new();
        let mut backups: Vec<(String, String)> = Vec::new();

        for (path, search, replace) in &edits {
            if Path::new(path).exists() {
                let content = fs::read_to_string(path)?;
                backups.push((path.clone(), content));
            }
        }

        let mut all_success = true;
        for (path, search, replace) in &edits {
            match skill.edit_file(path, search, replace) {
                Ok(result) => {
                    if !result.success {
                        all_success = false;
                    }
                    results.push(result);
                }
                Err(e) => {
                    all_success = false;
                    results.push(EditResult {
                        success: false,
                        file_path: path.clone(),
                        changes_made: 0,
                        message: e.to_string(),
                    });
                }
            }
        }

        if !all_success {
            for (path, content) in backups {
                let _ = fs::write(&path, &content);
            }
            for result in &mut results {
                if result.success {
                    result.success = false;
                    result.message = "Rolled back due to other failures".to_string();
                }
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_edit_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "hello world").unwrap();

        let skill = EditFileSkill::new();
        let result = skill
            .edit_file(file_path.to_str().unwrap(), "world", "Webrana")
            .unwrap();

        assert!(result.success);
        assert_eq!(result.changes_made, 1);

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "hello Webrana");
    }

    #[test]
    fn test_parse_diff() {
        let skill = EditFileSkill::new();
        let diff = r#"<<<<<<< SEARCH
old code
=======
new code
>>>>>>> REPLACE"#;

        let ops = skill.parse_diff(diff).unwrap();
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].search, "old code");
        assert_eq!(ops[0].replace, "new code");
    }
}
