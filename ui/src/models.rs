use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Response wrapper for commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

// Project models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
    pub settings: ProjectSettings,
    pub cc_definitions: HashMap<String, CCDefinition>,
    pub scenes: HashMap<String, Scene>,
    pub grid_assignments: HashMap<u8, String>, // Position -> SceneId (u8 key matches backend)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub default_output_device: Option<String>,
    pub default_controller_device: Option<String>,
    pub auto_connect: bool,
    pub default_tempo: f64,
    pub use_link: bool,
    pub default_quantization: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
    pub file_path: String,
}

// Scene models
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TriggerMode {
    Immediate,
    NextBeat,
    Beats(u8),
    NextBar,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Scene {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub trigger_mode: TriggerMode,
    pub cc_values: HashMap<String, CCValue>,
    pub tags: Vec<String>,
    pub active: bool,
    pub favorite: bool,
    pub grid_position: Option<u8>,
    pub color: Option<(u8, u8, u8)>,
}

// CC value models
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TransitionCurve {
    Linear,
    Exponential,
    Logarithmic,
    SCurve,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CCValue {
    pub channel: u8,
    pub cc_number: u8,
    pub value: u8,
    pub name: Option<String>,
    pub transition: bool,
    pub transition_beats: Option<f32>,
    pub transition_ms: Option<u32>,
    pub curve: TransitionCurve,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CCDefinition {
    pub cc_number: u8,
    pub channel: u8,
    pub name: String,
    pub description: Option<String>,
    pub min_value: u8,
    pub max_value: u8,
    pub default_value: u8,
    pub use_transitions: bool,
}

// MIDI device models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiDevice {
    pub id: String,
    pub name: String,
    pub is_input: bool,
    pub is_controller: bool,
}

// AI Generation models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationParams {
    pub description: String,
    pub cc_definitions: Vec<CCDefinitionRef>,
    pub previous_scene: Option<SceneRef>,
    pub use_transitions: bool,
    pub add_randomness: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CCDefinitionRef {
    pub channel: u8,
    pub cc_number: u8,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneRef {
    pub id: String,
    pub name: String,
    pub cc_values: HashMap<String, CCValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedScene {
    pub scene: Scene,
    pub explanation: String,
}