// Re-export all components
pub mod grid;
pub mod scene_editor;
pub mod cc_editor;
pub mod midi_monitor;
pub mod settings;
pub mod dialogs;

pub use grid::SceneGrid;
pub use scene_editor::SceneEditor;
pub use cc_editor::CCEditor;
pub use midi_monitor::MidiDeviceList;
pub use settings::SettingsPanel;
pub use dialogs::*;