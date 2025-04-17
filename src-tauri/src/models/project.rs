use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::cc::CCValue;
use crate::models::scene::Scene;

/// Metadata for a CC definition within a project
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CCDefinition {
    /// CC number (0-127)
    pub cc_number: u8,

    /// MIDI channel (0-15)
    pub channel: u8,

    /// Display name
    pub name: String,

    /// Description for AI assistance
    #[serde(default)]
    pub description: Option<String>,

    /// Minimum value (default: 0)
    #[serde(default)]
    pub min_value: u8,

    /// Maximum value (default: 127)
    #[serde(default = "default_max_value")]
    pub max_value: u8,

    /// Default value when adding to scenes
    #[serde(default = "default_cc_value")]
    pub default_value: u8,

    /// Whether this CC typically uses transitions
    #[serde(default)]
    pub use_transitions: bool,
}

fn default_max_value() -> u8 {
    127
}

fn default_cc_value() -> u8 {
    0
}

impl CCDefinition {
    /// Create a new CC definition
    pub fn new(channel: u8, cc_number: u8, name: &str) -> Self {
        CCDefinition {
            channel,
            cc_number,
            name: name.to_string(),
            description: None,
            min_value: 0,
            max_value: 127,
            default_value: 0,
            use_transitions: false,
        }
    }

    /// Create a CCValue from this definition
    pub fn create_cc_value(&self, value: Option<u8>) -> CCValue {
        let value = value.unwrap_or(self.default_value);
        let mut cc = CCValue::new(self.channel, self.cc_number, value);
        cc.name = Some(self.name.clone());
        cc.description = self.description.clone();
        cc.transition = self.use_transitions;
        cc
    }
}

/// Global project settings
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectSettings {
    /// Default MIDI output device
    #[serde(default)]
    pub default_output_device: Option<String>,

    /// Default controller device
    #[serde(default)]
    pub default_controller_device: Option<String>,

    /// Whether to auto-connect to the last used devices on startup
    #[serde(default)]
    pub auto_connect: bool,

    /// Default tempo when not synced to Link
    #[serde(default = "default_tempo")]
    pub default_tempo: f64,

    /// Whether to use Ableton Link by default
    #[serde(default)]
    pub use_link: bool,

    /// Default quantization (in beats) for scene transitions
    #[serde(default)]
    pub default_quantization: Option<u8>,
}

fn default_tempo() -> f64 {
    120.0
}

impl Default for ProjectSettings {
    fn default() -> Self {
        ProjectSettings {
            default_output_device: None,
            default_controller_device: None,
            auto_connect: false,
            default_tempo: default_tempo(),
            use_link: false,
            default_quantization: None,
        }
    }
}

/// Complete project containing scenes and settings
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    /// Unique project ID
    pub id: String,

    /// Project name
    pub name: String,

    /// Project description
    #[serde(default)]
    pub description: Option<String>,

    /// Project author
    #[serde(default)]
    pub author: Option<String>,

    /// Project version
    pub version: String,

    /// Creation timestamp
    pub created_at: String,

    /// Last modified timestamp
    pub updated_at: String,

    /// Project settings
    #[serde(default)]
    pub settings: ProjectSettings,

    /// CC definitions
    pub cc_definitions: HashMap<String, CCDefinition>,

    /// Scenes in this project
    pub scenes: HashMap<String, Scene>,

    /// Grid assignments (maps grid position to scene id)
    pub grid_assignments: HashMap<u8, String>,
}

impl Project {
    /// Create a new project
    pub fn new(name: &str, author: Option<&str>) -> Self {
        let now = chrono::Utc::now().to_rfc3339();

        Project {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: None,
            author: author.map(|s| s.to_string()),
            version: "1.0.0".to_string(),
            created_at: now.clone(),
            updated_at: now,
            settings: ProjectSettings::default(),
            cc_definitions: HashMap::new(),
            scenes: HashMap::new(),
            grid_assignments: HashMap::new(),
        }
    }

    /// Add a CC definition
    pub fn add_cc_definition(&mut self, definition: CCDefinition) -> &mut Self {
        let key = format!("{}:{}", definition.channel, definition.cc_number);
        self.cc_definitions.insert(key, definition);
        self
    }

    /// Get a CC definition
    pub fn get_cc_definition(&self, channel: u8, cc_number: u8) -> Option<&CCDefinition> {
        let key = format!("{}:{}", channel, cc_number);
        self.cc_definitions.get(&key)
    }

    /// Add a scene
    pub fn add_scene(&mut self, scene: Scene) -> &mut Self {
        self.scenes.insert(scene.id.clone(), scene);
        self
    }

    /// Get a scene by ID
    pub fn get_scene(&self, id: &str) -> Option<&Scene> {
        self.scenes.get(id)
    }

    /// Get a mutable reference to a scene by ID
    pub fn get_scene_mut(&mut self, id: &str) -> Option<&mut Scene> {
        self.scenes.get_mut(id)
    }

    /// Remove a scene
    pub fn remove_scene(&mut self, id: &str) -> Option<Scene> {
        // Remove from grid assignments first
        self.grid_assignments.retain(|_, scene_id| scene_id != id);

        // Remove the scene
        self.scenes.remove(id)
    }

    /// Assign a scene to a grid position
    pub fn assign_to_grid(&mut self, scene_id: &str, position: u8) -> Result<(), String> {
        if position >= 64 {
            return Err("Grid position must be between 0 and 63".to_string());
        }

        if !self.scenes.contains_key(scene_id) {
            return Err(format!("Scene with ID {} not found", scene_id));
        }

        // Update the scene's grid position
        if let Some(scene) = self.scenes.get_mut(scene_id) {
            scene.grid_position = Some(position);
        }

        // Update the grid assignment
        self.grid_assignments.insert(position, scene_id.to_string());

        Ok(())
    }

    /// Get the scene assigned to a grid position
    pub fn get_scene_at_grid(&self, position: u8) -> Option<&Scene> {
        if position >= 64 {
            return None;
        }

        self.grid_assignments
            .get(&position)
            .and_then(|id| self.scenes.get(id))
    }

    /// Update the last modified timestamp
    pub fn update_timestamp(&mut self) {
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// Create a duplicate of this project with a new ID and name
    pub fn duplicate(&self, new_name: &str) -> Self {
        let mut new_project = self.clone();
        new_project.id = Uuid::new_v4().to_string();
        new_project.name = new_name.to_string();
        new_project.created_at = chrono::Utc::now().to_rfc3339();
        new_project.updated_at = new_project.created_at.clone();
        new_project
    }
}

/// Tests for Project
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_creation() {
        let project = Project::new("Test Project", Some("Test Author"));

        assert_eq!(project.name, "Test Project");
        assert_eq!(project.author, Some("Test Author".to_string()));
        assert_eq!(project.scenes.len(), 0);
        assert_eq!(project.cc_definitions.len(), 0);
    }

    #[test]
    fn test_add_scene() {
        let mut project = Project::new("Test Project", None);

        let scene = Scene::new("scene-1", "Test Scene");
        project.add_scene(scene);

        assert_eq!(project.scenes.len(), 1);
        assert!(project.get_scene("scene-1").is_some());
    }

    #[test]
    fn test_grid_assignment() {
        let mut project = Project::new("Test Project", None);

        let scene = Scene::new("scene-1", "Test Scene");
        project.add_scene(scene);

        let result = project.assign_to_grid("scene-1", 5);
        assert!(result.is_ok());

        let scene_at_grid = project.get_scene_at_grid(5);
        assert!(scene_at_grid.is_some());
        assert_eq!(scene_at_grid.unwrap().id, "scene-1");

        // Test invalid position
        let result = project.assign_to_grid("scene-1", 100);
        assert!(result.is_err());

        // Test invalid scene ID
        let result = project.assign_to_grid("non-existent", 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_cc_definitions() {
        let mut project = Project::new("Test Project", None);

        let cc_def = CCDefinition::new(0, 1, "Test CC");
        project.add_cc_definition(cc_def);

        assert_eq!(project.cc_definitions.len(), 1);

        let retrieved = project.get_cc_definition(0, 1);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test CC");

        // Test creating CC value from definition
        let cc_value = retrieved.unwrap().create_cc_value(Some(64));
        assert_eq!(cc_value.value, 64);
        assert_eq!(cc_value.name, Some("Test CC".to_string()));
    }
}
