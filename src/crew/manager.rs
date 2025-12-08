//! Crew Manager - Create, list, and manage crew members

use super::{Crew, CrewTemplate};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Manages crew members (custom AI personas)
pub struct CrewManager {
    /// Directory storing crew definitions
    crew_dir: PathBuf,
    
    /// Loaded crew members
    crews: HashMap<String, Crew>,
    
    /// Currently active crew
    active_crew: Option<String>,
}

impl CrewManager {
    /// Create a new crew manager with default directory
    pub fn new() -> Result<Self> {
        let crew_dir = Self::default_crew_dir()?;
        Self::with_dir(crew_dir)
    }

    /// Create a crew manager with custom directory
    pub fn with_dir(crew_dir: PathBuf) -> Result<Self> {
        // Create directory if it doesn't exist
        if !crew_dir.exists() {
            fs::create_dir_all(&crew_dir)?;
        }

        let mut manager = Self {
            crew_dir,
            crews: HashMap::new(),
            active_crew: None,
        };

        // Load existing crews
        manager.load_all()?;

        Ok(manager)
    }

    /// Get default crew directory
    fn default_crew_dir() -> Result<PathBuf> {
        let dir = directories::ProjectDirs::from("dev", "webrana", "webrana-cli")
            .map(|dirs| dirs.data_dir().join("crew"))
            .unwrap_or_else(|| PathBuf::from(".webrana/crew"));
        Ok(dir)
    }

    /// Load all crews from disk
    fn load_all(&mut self) -> Result<()> {
        if !self.crew_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&self.crew_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map(|e| e == "yaml" || e == "yml").unwrap_or(false) {
                if let Ok(crew) = self.load_crew_file(&path) {
                    self.crews.insert(crew.id.clone(), crew);
                }
            }
        }

        // Load active crew from state file
        let state_file = self.crew_dir.join(".active");
        if state_file.exists() {
            if let Ok(active) = fs::read_to_string(&state_file) {
                let active = active.trim().to_string();
                if self.crews.contains_key(&active) {
                    self.active_crew = Some(active);
                }
            }
        }

        Ok(())
    }

    /// Load a single crew file
    fn load_crew_file(&self, path: &Path) -> Result<Crew> {
        let content = fs::read_to_string(path)?;
        let crew: Crew = serde_yaml::from_str(&content)?;
        Ok(crew)
    }

    /// Save a crew to disk
    fn save_crew(&self, crew: &Crew) -> Result<()> {
        let path = self.crew_dir.join(format!("{}.yaml", crew.id));
        let content = serde_yaml::to_string(crew)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Create a new crew member
    pub fn create(&mut self, crew: Crew) -> Result<()> {
        if self.crews.contains_key(&crew.id) {
            return Err(anyhow!("Crew '{}' already exists", crew.id));
        }

        self.save_crew(&crew)?;
        self.crews.insert(crew.id.clone(), crew);
        Ok(())
    }

    /// Create from template
    pub fn create_from_template(&mut self, template: CrewTemplate) -> Result<Crew> {
        let crew = template.create();
        
        if self.crews.contains_key(&crew.id) {
            return Err(anyhow!("Crew '{}' already exists", crew.id));
        }

        self.save_crew(&crew)?;
        self.crews.insert(crew.id.clone(), crew.clone());
        Ok(crew)
    }

    /// Get a crew by ID
    pub fn get(&self, id: &str) -> Option<&Crew> {
        self.crews.get(id)
    }

    /// Get mutable reference to crew
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Crew> {
        self.crews.get_mut(id)
    }

    /// List all crews
    pub fn list(&self) -> Vec<&Crew> {
        self.crews.values().collect()
    }

    /// Delete a crew
    pub fn delete(&mut self, id: &str) -> Result<bool> {
        if let Some(_crew) = self.crews.remove(id) {
            let path = self.crew_dir.join(format!("{}.yaml", id));
            if path.exists() {
                fs::remove_file(path)?;
            }
            
            // Clear active if it was this crew
            if self.active_crew.as_deref() == Some(id) {
                self.active_crew = None;
                let state_file = self.crew_dir.join(".active");
                let _ = fs::remove_file(state_file);
            }
            
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Set active crew
    pub fn set_active(&mut self, id: &str) -> Result<()> {
        if !self.crews.contains_key(id) {
            return Err(anyhow!("Crew '{}' not found", id));
        }

        self.active_crew = Some(id.to_string());
        
        // Persist active state
        let state_file = self.crew_dir.join(".active");
        fs::write(state_file, id)?;
        
        Ok(())
    }

    /// Clear active crew
    pub fn clear_active(&mut self) -> Result<()> {
        self.active_crew = None;
        let state_file = self.crew_dir.join(".active");
        if state_file.exists() {
            fs::remove_file(state_file)?;
        }
        Ok(())
    }

    /// Get active crew
    pub fn active(&self) -> Option<&Crew> {
        self.active_crew.as_ref().and_then(|id| self.crews.get(id))
    }

    /// Get active crew ID
    pub fn active_id(&self) -> Option<&str> {
        self.active_crew.as_deref()
    }

    /// Update a crew
    pub fn update(&mut self, crew: Crew) -> Result<()> {
        if !self.crews.contains_key(&crew.id) {
            return Err(anyhow!("Crew '{}' not found", crew.id));
        }

        self.save_crew(&crew)?;
        self.crews.insert(crew.id.clone(), crew);
        Ok(())
    }

    /// Export crew to YAML string
    pub fn export(&self, id: &str) -> Result<String> {
        let crew = self.get(id).ok_or_else(|| anyhow!("Crew '{}' not found", id))?;
        let yaml = serde_yaml::to_string(crew)?;
        Ok(yaml)
    }

    /// Import crew from YAML string
    pub fn import(&mut self, yaml: &str) -> Result<Crew> {
        let crew: Crew = serde_yaml::from_str(yaml)?;
        
        if self.crews.contains_key(&crew.id) {
            return Err(anyhow!("Crew '{}' already exists", crew.id));
        }

        self.save_crew(&crew)?;
        self.crews.insert(crew.id.clone(), crew.clone());
        Ok(crew)
    }

    /// Get crew directory path
    pub fn crew_dir(&self) -> &Path {
        &self.crew_dir
    }

    /// Count crews
    pub fn count(&self) -> usize {
        self.crews.len()
    }
}

impl Default for CrewManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            crew_dir: PathBuf::from(".webrana/crew"),
            crews: HashMap::new(),
            active_crew: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_crew_manager() {
        let tmp = TempDir::new().unwrap();
        let mut manager = CrewManager::with_dir(tmp.path().to_path_buf()).unwrap();

        // Create crew
        let crew = Crew::new("test", "Test Crew", "A test crew", "You are a test.");
        manager.create(crew).unwrap();

        // List
        assert_eq!(manager.count(), 1);
        assert!(manager.get("test").is_some());

        // Set active
        manager.set_active("test").unwrap();
        assert_eq!(manager.active_id(), Some("test"));

        // Delete
        manager.delete("test").unwrap();
        assert_eq!(manager.count(), 0);
        assert!(manager.active_id().is_none());
    }

    #[test]
    fn test_template_creation() {
        let tmp = TempDir::new().unwrap();
        let mut manager = CrewManager::with_dir(tmp.path().to_path_buf()).unwrap();

        let crew = manager.create_from_template(CrewTemplate::CodeReviewer).unwrap();
        assert_eq!(crew.id, "code-reviewer");
        assert!(manager.get("code-reviewer").is_some());
    }

    #[test]
    fn test_import_export() {
        let tmp = TempDir::new().unwrap();
        let mut manager = CrewManager::with_dir(tmp.path().to_path_buf()).unwrap();

        let crew = Crew::new("export-test", "Export Test", "Test", "Prompt");
        manager.create(crew).unwrap();

        let yaml = manager.export("export-test").unwrap();
        assert!(yaml.contains("export-test"));

        // Import to new manager
        let tmp2 = TempDir::new().unwrap();
        let mut manager2 = CrewManager::with_dir(tmp2.path().to_path_buf()).unwrap();
        manager2.import(&yaml).unwrap();
        assert!(manager2.get("export-test").is_some());
    }
}
