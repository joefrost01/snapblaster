use std::path::Path;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use crate::models::project::Project;
use crate::models::scene::Scene;
use crate::models::cc::CCValue;
use crate::project::storage::{ProjectStorage, StorageError, ProjectMeta};
use crate::midi::engine::{MidiEngine, MidiCommand};
use crate::midi::devices::{DeviceRegistry, MidiDevice};
use crate::midi::controller::{GridController, ControllerEvent, Color};

/// Errors specific to project management
#[derive(Debug)]
pub enum ProjectManagerError {
    StorageError(StorageError),
    MidiError(String),
    InvalidSceneId(String),
    InvalidGridPosition(u8),
    NoActiveProject,
    NoActiveScene,
    NoAvailableDevices,
}

impl From<StorageError> for ProjectManagerError {
    fn from(error: StorageError) -> Self {
        ProjectManagerError::StorageError(error)
    }
}

impl From<String> for ProjectManagerError {
    fn from(error: String) -> Self {
        ProjectManagerError::MidiError(error)
    }
}

impl std::fmt::Display for ProjectManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectManagerError::StorageError(e) => write!(f, "Storage error: {}", e),
            ProjectManagerError::MidiError(e) => write!(f, "MIDI error: {}", e),
            ProjectManagerError::InvalidSceneId(id) => write!(f, "Invalid scene ID: {}", id),
            ProjectManagerError::InvalidGridPosition(pos) => write!(f, "Invalid grid position: {}", pos),
            ProjectManagerError::NoActiveProject => write!(f, "No active project"),
            ProjectManagerError::NoActiveScene => write!(f, "No active scene"),
            ProjectManagerError::NoAvailableDevices => write!(f, "No available MIDI devices"),
        }
    }
}

impl std::error::Error for ProjectManagerError {}

type Result<T> = std::result::Result<T, ProjectManagerError>;

/// Manager for projects and scenes
pub struct ProjectManager {
    storage: ProjectStorage,
    pub(crate) device_registry: Arc<DeviceRegistry>,
    midi_engine: Arc<Mutex<MidiEngine>>,
    active_project: Arc<Mutex<Option<Project>>>,
    active_scene_id: Arc<Mutex<Option<String>>>,
    controller: Arc<Mutex<Option<Box<dyn GridController>>>>,
}

impl ProjectManager {
    /// Create a new project manager
    pub fn new(
        storage: ProjectStorage,
        device_registry: Arc<DeviceRegistry>,
        midi_engine: Arc<Mutex<MidiEngine>>,
    ) -> Self {
        ProjectManager {
            storage,
            device_registry,
            midi_engine,
            active_project: Arc::new(Mutex::new(None)),
            active_scene_id: Arc::new(Mutex::new(None)),
            controller: Arc::new(Mutex::new(None)),
        }
    }

    /// Initialize the project manager with default settings
    pub fn init() -> Result<Self> {
        let storage = ProjectStorage::init().map_err(ProjectManagerError::StorageError)?;
        let device_registry = Arc::new(DeviceRegistryFactory::create()?);
        let midi_engine = Arc::new(Mutex::new(MidiEngine::new()?));

        // Start the MIDI engine
        midi_engine.lock().unwrap().start()?;

        Ok(ProjectManager::new(storage, device_registry, midi_engine))
    }

    /// Create a new project
    pub fn create_project(&self, name: &str, author: Option<&str>) -> Result<String> {
        let project = Project::new(name, author);
        let id = project.id.clone();

        self.storage.save_project(&project)?;

        // Set as active project
        let mut active_project = self.active_project.lock().unwrap();
        *active_project = Some(project);

        Ok(id)
    }

    /// Create a new project from template
    pub fn create_from_template(&self, name: &str, author: Option<&str>) -> Result<String> {
        let project = self.storage.create_template_project(name, author);
        let id = project.id.clone();

        self.storage.save_project(&project)?;

        // Set as active project
        let mut active_project = self.active_project.lock().unwrap();
        *active_project = Some(project);

        Ok(id)
    }

    /// Get a list of all projects
    pub fn list_projects(&self) -> Result<Vec<ProjectMeta>> {
        self.storage.list_projects().map_err(ProjectManagerError::StorageError)
    }

    /// Load a project and set it as active
    pub fn load_project(&self, id: &str) -> Result<Project> {
        let project = self.storage.load_project(id)?;

        // Set as active project
        let mut active_project = self.active_project.lock().unwrap();
        *active_project = Some(project.clone());

        // Clear active scene
        let mut active_scene_id = self.active_scene_id.lock().unwrap();
        *active_scene_id = None;

        // Update controller grid
        self.update_controller_grid()?;

        Ok(project)
    }

    /// Get the active project
    pub fn get_active_project(&self) -> Result<Project> {
        let active_project = self.active_project.lock().unwrap();

        match &*active_project {
            Some(project) => Ok(project.clone()),
            None => Err(ProjectManagerError::NoActiveProject),
        }
    }

    /// Save the active project
    pub fn save_active_project(&self) -> Result<()> {
        let active_project = self.active_project.lock().unwrap();

        match &*active_project {
            Some(project) => {
                self.storage.save_project(project)?;
                Ok(())
            },
            None => Err(ProjectManagerError::NoActiveProject),
        }
    }

    /// Create a new scene in the active project
    pub fn create_scene(&self, name: &str, description: Option<&str>) -> Result<String> {
        let mut active_project = self.active_project.lock().unwrap();

        match &mut *active_project {
            Some(project) => {
                use uuid::Uuid;
                let id = Uuid::new_v4().to_string();

                let mut scene = Scene::new(&id, name);
                if let Some(desc) = description {
                    scene.description = Some(desc.to_string());
                }

                project.add_scene(scene);
                project.update_timestamp();

                self.storage.save_project(project)?;

                Ok(id)
            },
            None => Err(ProjectManagerError::NoActiveProject),
        }
    }

    /// Activate a scene
    pub fn activate_scene(&self, scene_id: &str) -> Result<()> {
        let active_project = self.active_project.lock().unwrap();

        match &*active_project {
            Some(project) => {
                // Find the scene
                let scene = project.get_scene(scene_id)
                    .ok_or_else(|| ProjectManagerError::InvalidSceneId(scene_id.to_string()))?;

                // Activate the scene via MIDI engine
                let midi_engine = self.midi_engine.lock().unwrap();

                let quantize_beats = if scene.trigger_mode != crate::models::scene::TriggerMode::Immediate {
                    Some(match scene.trigger_mode {
                        crate::models::scene::TriggerMode::NextBeat => 1,
                        crate::models::scene::TriggerMode::Beats(n) => n,
                        crate::models::scene::TriggerMode::NextBar => 4, // Assuming 4 beats per bar
                        _ => 0,
                    })
                } else {
                    None
                };

                midi_engine.send_command(MidiCommand::ActivateScene {
                    scene: scene.clone(),
                    quantize_beats,
                })?;

                // Set as active scene
                let mut active_scene_id = self.active_scene_id.lock().unwrap();
                *active_scene_id = Some(scene_id.to_string());

                Ok(())
            },
            None => Err(ProjectManagerError::NoActiveProject),
        }
    }

    /// Get the active scene
    pub fn get_active_scene(&self) -> Result<Scene> {
        let active_project = self.active_project.lock().unwrap();
        let active_scene_id = self.active_scene_id.lock().unwrap();

        match (&*active_project, &*active_scene_id) {
            (Some(project), Some(scene_id)) => {
                // Find the scene
                let scene = project.get_scene(scene_id)
                    .ok_or_else(|| ProjectManagerError::InvalidSceneId(scene_id.to_string()))?;

                Ok(scene.clone())
            },
            (None, _) => Err(ProjectManagerError::NoActiveProject),
            (_, None) => Err(ProjectManagerError::NoActiveScene),
        }
    }

    /// Assign a scene to a grid position
    pub fn assign_scene_to_grid(&self, scene_id: &str, position: u8) -> Result<()> {
        if position >= 64 {
            return Err(ProjectManagerError::InvalidGridPosition(position));
        }

        let mut active_project = self.active_project.lock().unwrap();

        match &mut *active_project {
            Some(project) => {
                project.assign_to_grid(scene_id, position)?;
                project.update_timestamp();

                self.storage.save_project(project)?;

                // Update controller if connected
                self.update_controller_grid()?;

                Ok(())
            },
            None => Err(ProjectManagerError::NoActiveProject),
        }
    }

    /// Connect to a MIDI controller
    pub fn connect_controller(&self, device_id: &str) -> Result<()> {
        // Find the device
        let device = self.device_registry.get_device(device_id)
            .ok_or_else(|| ProjectManagerError::MidiError(format!("Device not found: {}", device_id)))?;

        // Create the controller
        let mut controller = crate::midi::controller::ControllerFactory::create_controller(device)?;

        // Connect to the controller
        controller.connect()?;

        // Register event callback
        let active_project = Arc::clone(&self.active_project);
        let this = Arc::new(self.clone());

        // Clone references needed for the callback instead of the controller itself
        let controller_ref = Arc::new(Mutex::new(controller.clone_box())); // Use clone_box here

        controller.set_event_callback(Arc::new(move |event| {
            match event {
                ControllerEvent::PadPressed(grid_id, _) => {
                    // Get access to controller clone
                    if let Ok(controller_guard) = controller_ref.lock() {
                        // Translate controller grid ID to app grid ID
                        if let Some(app_grid_id) = controller_guard.map_to_app_grid_id(grid_id) {
                            // Find scene at this grid position
                            let active_project_guard = active_project.lock().unwrap();

                            if let Some(project) = &*active_project_guard {
                                if let Some(scene) = project.get_scene_at_grid(app_grid_id) {
                                    // Activate the scene
                                    let scene_id = scene.id.clone();
                                    drop(active_project_guard); // Release the lock

                                    let _ = this.activate_scene(&scene_id);
                                }
                            }
                        }
                    }
                },
                _ => {} // Ignore other events for now
            }
        }));

        // Store the controller
        let mut controller_guard = self.controller.lock().unwrap();
        *controller_guard = Some(controller);

        // Update the controller grid
        self.update_controller_grid()?;

        Ok(())
    }

    /// Update the controller grid with current scene assignments
    fn update_controller_grid(&self) -> Result<()> {
        let active_project = self.active_project.lock().unwrap();
        let mut controller_guard = self.controller.lock().unwrap();

        if let (Some(project), Some(controller)) = (&*active_project, &mut *controller_guard) {
            // Clear the grid
            controller.clear()?;

            // Set colors for assigned scenes
            for (&position, scene_id) in &project.grid_assignments {
                if let Some(scene) = project.get_scene(scene_id) {
                    // Default color if not specified
                    let color = scene.color
                        .map(|(r, g, b)| Color::new(r, g, b))
                        .unwrap_or(Color::GREEN);

                    // Map from app grid ID to controller-specific ID
                    let controller_id = controller.map_grid_id(position);
                    controller.set_pad_color(controller_id, color)?;
                }
            }
        }

        Ok(())
    }

    /// Disconnect from the controller
    pub fn disconnect_controller(&self) -> Result<()> {
        let mut controller_guard = self.controller.lock().unwrap();

        if let Some(controller) = &mut *controller_guard {
            controller.disconnect()?;
            *controller_guard = None;
        }

        Ok(())
    }

    /// Send a single CC value
    pub fn send_cc(&self, channel: u8, cc_number: u8, value: u8) -> Result<()> {
        let midi_engine = self.midi_engine.lock().unwrap();

        midi_engine.send_command(MidiCommand::SendCC {
            channel,
            cc_number,
            value,
        })?;

        Ok(())
    }

    /// Import a project from a file
    pub fn import_project<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        let project = self.storage.import_project(path)?;
        let id = project.id.clone();

        // Set as active project
        let mut active_project = self.active_project.lock().unwrap();
        *active_project = Some(project);

        Ok(id)
    }

    /// Export the active project to a file
    pub fn export_active_project<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let active_project = self.active_project.lock().unwrap();

        match &*active_project {
            Some(project) => {
                self.storage.export_project(&project.id, path)?;
                Ok(())
            },
            None => Err(ProjectManagerError::NoActiveProject),
        }
    }

    /// Close the active project
    pub fn close_active_project(&self) -> Result<()> {
        let mut active_project = self.active_project.lock().unwrap();
        let mut active_scene_id = self.active_scene_id.lock().unwrap();

        *active_project = None;
        *active_scene_id = None;

        // Clear controller grid
        if let Ok(mut controller_guard) = self.controller.lock() {
            if let Some(controller) = &mut *controller_guard {
                let _ = controller.clear();
            }
        }

        Ok(())
    }

    pub fn get_midi_devices() -> Result<Vec<MidiDevice>> {
        let device_registry = DeviceRegistryFactory::create()
            .map_err(|e| e.to_string())?;

        Ok(device_registry.get_all_devices())
    }
}

impl Clone for ProjectManager {
    fn clone(&self) -> Self {
        ProjectManager {
            storage: ProjectStorage::init().unwrap(),
            device_registry: Arc::clone(&self.device_registry),
            midi_engine: Arc::clone(&self.midi_engine),
            active_project: Arc::clone(&self.active_project),
            active_scene_id: Arc::clone(&self.active_scene_id),
            controller: Arc::clone(&self.controller),
        }
    }
}

// Factory for DeviceRegistry to fix compiler issues
pub struct DeviceRegistryFactory;

impl DeviceRegistryFactory {
    pub fn create() -> Result<DeviceRegistry> {
        crate::midi::devices::DeviceRegistryFactory::create()
            .map_err(|e| ProjectManagerError::MidiError(e))
    }
}