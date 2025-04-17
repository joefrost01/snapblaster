use crate::models::cc::CCValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Definition of how a scene is triggered
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TriggerMode {
    /// Trigger immediately
    Immediate,
    /// Trigger on next beat
    NextBeat,
    /// Trigger on next N beats
    Beats(u8),
    /// Trigger on next bar
    NextBar,
}

impl Default for TriggerMode {
    fn default() -> Self {
        TriggerMode::Immediate
    }
}

/// A scene containing a collection of CC values
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Scene {
    /// Unique identifier for the scene
    pub id: String,

    /// Display name
    pub name: String,

    /// Brief description of the scene
    #[serde(default)]
    pub description: Option<String>,

    /// How the scene should be triggered
    #[serde(default)]
    pub trigger_mode: TriggerMode,

    /// CC values in this scene
    pub cc_values: HashMap<String, CCValue>,

    /// Tags for organization and AI assistance
    #[serde(default)]
    pub tags: Vec<String>,

    /// Whether this scene is active
    #[serde(default)]
    pub active: bool,

    /// Whether this scene is a favorite
    #[serde(default)]
    pub favorite: bool,

    /// The grid position (0-63) where this scene is assigned
    #[serde(default)]
    pub grid_position: Option<u8>,

    /// RGB color for this scene (for display on hardware)
    #[serde(default)]
    pub color: Option<(u8, u8, u8)>,
}

impl Scene {
    /// Create a new scene with default values
    pub fn new(id: &str, name: &str) -> Self {
        Scene {
            id: id.to_string(),
            name: name.to_string(),
            description: None,
            trigger_mode: TriggerMode::default(),
            cc_values: HashMap::new(),
            tags: Vec::new(),
            active: false,
            favorite: false,
            grid_position: None,
            color: None,
        }
    }

    /// Add a CC value to the scene
    pub fn add_cc(&mut self, cc: CCValue) -> &mut Self {
        let key = format!("{}:{}", cc.channel, cc.cc_number);
        self.cc_values.insert(key, cc);
        self
    }

    /// Add multiple CC values to the scene
    pub fn add_cc_values(&mut self, cc_values: Vec<CCValue>) -> &mut Self {
        for cc in cc_values {
            self.add_cc(cc);
        }
        self
    }

    /// Get a CC value by channel and number
    pub fn get_cc(&self, channel: u8, cc_number: u8) -> Option<&CCValue> {
        let key = format!("{}:{}", channel, cc_number);
        self.cc_values.get(&key)
    }

    /// Remove a CC value
    pub fn remove_cc(&mut self, channel: u8, cc_number: u8) -> Option<CCValue> {
        let key = format!("{}:{}", channel, cc_number);
        self.cc_values.remove(&key)
    }

    /// Set the scene's grid position and color
    pub fn set_grid_position(&mut self, position: u8, color: Option<(u8, u8, u8)>) -> &mut Self {
        if position < 64 {
            self.grid_position = Some(position);
            self.color = color;
        }
        self
    }

    /// Get all CC values as a vector
    pub fn cc_values_vec(&self) -> Vec<&CCValue> {
        self.cc_values.values().collect()
    }

    /// Get a deep copy of this scene with a new ID and name
    pub fn duplicate(&self, new_id: &str, new_name: &str) -> Self {
        let mut new_scene = self.clone();
        new_scene.id = new_id.to_string();
        new_scene.name = new_name.to_string();
        new_scene.active = false;
        new_scene.grid_position = None;
        new_scene
    }

    /// Set the trigger mode
    pub fn with_trigger_mode(mut self, mode: TriggerMode) -> Self {
        self.trigger_mode = mode;
        self
    }

    /// Add tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Set description
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
}

/// Tests for Scene
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_creation() {
        let scene = Scene::new("test-scene", "Test Scene");
        assert_eq!(scene.id, "test-scene");
        assert_eq!(scene.name, "Test Scene");
        assert_eq!(scene.cc_values.len(), 0);
    }

    #[test]
    fn test_adding_cc_values() {
        let mut scene = Scene::new("test-scene", "Test Scene");

        let cc1 = CCValue::new(0, 1, 64);
        let cc2 = CCValue::new(0, 2, 127);

        scene.add_cc(cc1);
        scene.add_cc(cc2);

        assert_eq!(scene.cc_values.len(), 2);
        assert_eq!(scene.get_cc(0, 1).unwrap().value, 64);
        assert_eq!(scene.get_cc(0, 2).unwrap().value, 127);
    }

    #[test]
    fn test_removing_cc_values() {
        let mut scene = Scene::new("test-scene", "Test Scene");

        let cc1 = CCValue::new(0, 1, 64);
        let cc2 = CCValue::new(0, 2, 127);

        scene.add_cc(cc1);
        scene.add_cc(cc2);

        let removed = scene.remove_cc(0, 1);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().value, 64);
        assert_eq!(scene.cc_values.len(), 1);
        assert!(scene.get_cc(0, 1).is_none());
    }

    #[test]
    fn test_grid_position() {
        let mut scene = Scene::new("test-scene", "Test Scene");
        scene.set_grid_position(42, Some((255, 0, 127)));

        assert_eq!(scene.grid_position, Some(42));
        assert_eq!(scene.color, Some((255, 0, 127)));

        // Test with invalid position
        scene.set_grid_position(200, Some((0, 255, 0)));
        assert_eq!(scene.grid_position, Some(42)); // Unchanged
    }
}
