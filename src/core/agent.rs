use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub name: String,
    pub description: String,
    pub system_prompt: String,
    pub model: String,
    pub skills: Vec<String>,
    pub temperature: f32,
}

impl Agent {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        system_prompt: impl Into<String>,
        model: impl Into<String>,
        skills: Vec<String>,
        temperature: f32,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            system_prompt: system_prompt.into(),
            model: model.into(),
            skills,
            temperature,
        }
    }

    pub fn can_use_skill(&self, skill: &str) -> bool {
        self.skills.contains(&"*".to_string()) || self.skills.contains(&skill.to_string())
    }
}
