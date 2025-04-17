use serde::Serialize;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{command, State};

use crate::ai::generator::{GeneratedScene, GenerationParams, SceneGenerator};
use crate::midi::devices::MidiDevice;
use crate::models::project::Project;
use crate::models::scene::Scene;
use crate::project::manager::ProjectManager;
use crate::project::storage::ProjectMeta;

/// App state containing shared resources
pub struct AppState {
    pub project_manager: Arc<Mutex<ProjectManager>>,
    pub scene_generator: Arc<Mutex<SceneGenerator>>,
}

/// Response containing a result or error
#[derive(Serialize)]
pub struct CommandResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> CommandResponse<T> {
    pub fn success(data: T) -> Self {
        CommandResponse {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: &str) -> Self {
        CommandResponse {
            success: false,
            data: None,
            error: Some(message.to_string()),
        }
    }
}

/// Project management commands

#[tauri::command]
pub async fn list_projects(state: State<'_, AppState>) -> Result<Vec<ProjectMeta>, String> {
    let project_manager = state.project_manager.lock().unwrap();

    project_manager
        .list_projects()
        .map_err(|e| format!("Failed to list projects: {}", e))
}

#[tauri::command]
pub async fn get_active_project(
    state: State<'_, AppState>,
) -> Result<CommandResponse<Project>, String> {
    let project_manager = state.project_manager.lock().unwrap();

    match project_manager.get_active_project() {
        Ok(project) => Ok(CommandResponse::success(project)),
        Err(e) => Ok(CommandResponse::error(&format!("No active project: {}", e))),
    }
}

#[tauri::command]
pub async fn create_project(
    name: String,
    author: Option<String>,
    state: State<'_, AppState>,
) -> Result<CommandResponse<String>, String> {
    let mut project_manager = state.project_manager.lock().unwrap();

    match project_manager.create_project(&name, author.as_deref()) {
        Ok(id) => Ok(CommandResponse::success(id)),
        Err(e) => Ok(CommandResponse::error(&format!(
            "Failed to create project: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn create_template_project(
    name: String,
    author: Option<String>,
    state: State<'_, AppState>,
) -> Result<CommandResponse<String>, String> {
    let mut project_manager = state.project_manager.lock().unwrap();

    match project_manager.create_from_template(&name, author.as_deref()) {
        Ok(id) => Ok(CommandResponse::success(id)),
        Err(e) => Ok(CommandResponse::error(&format!(
            "Failed to create template project: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn load_project(
    id: String,
    state: State<'_, AppState>,
) -> Result<CommandResponse<Project>, String> {
    let mut project_manager = state.project_manager.lock().unwrap();

    match project_manager.load_project(&id) {
        Ok(project) => Ok(CommandResponse::success(project)),
        Err(e) => Ok(CommandResponse::error(&format!(
            "Failed to load project: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn save_project(state: State<'_, AppState>) -> Result<CommandResponse<bool>, String> {
    let project_manager = state.project_manager.lock().unwrap();

    match project_manager.save_active_project() {
        Ok(_) => Ok(CommandResponse::success(true)),
        Err(e) => Ok(CommandResponse::error(&format!(
            "Failed to save project: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn import_project(
    path: String,
    state: State<'_, AppState>,
) -> Result<CommandResponse<String>, String> {
    let mut project_manager = state.project_manager.lock().unwrap();

    match project_manager.import_project(PathBuf::from(path)) {
        Ok(id) => Ok(CommandResponse::success(id)),
        Err(e) => Ok(CommandResponse::error(&format!(
            "Failed to import project: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn export_project(
    path: String,
    state: State<'_, AppState>,
) -> Result<CommandResponse<bool>, String> {
    let project_manager = state.project_manager.lock().unwrap();

    match project_manager.export_active_project(PathBuf::from(path)) {
        Ok(_) => Ok(CommandResponse::success(true)),
        Err(e) => Ok(CommandResponse::error(&format!(
            "Failed to export project: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn close_project(state: State<'_, AppState>) -> Result<CommandResponse<bool>, String> {
    let mut project_manager = state.project_manager.lock().unwrap();

    match project_manager.close_active_project() {
        Ok(_) => Ok(CommandResponse::success(true)),
        Err(e) => Ok(CommandResponse::error(&format!(
            "Failed to close project: {}",
            e
        ))),
    }
}

/// Scene management commands

#[tauri::command]
pub async fn create_scene(
    name: String,
    description: Option<String>,
    state: State<'_, AppState>,
) -> Result<CommandResponse<String>, String> {
    let mut project_manager = state.project_manager.lock().unwrap();

    match project_manager.create_scene(&name, description.as_deref()) {
        Ok(id) => Ok(CommandResponse::success(id)),
        Err(e) => Ok(CommandResponse::error(&format!(
            "Failed to create scene: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn get_scene(
    id: String,
    state: State<'_, AppState>,
) -> Result<CommandResponse<Scene>, String> {
    let project_manager = state.project_manager.lock().unwrap();

    match project_manager.get_active_project() {
        Ok(project) => match project.get_scene(&id) {
            Some(scene) => Ok(CommandResponse::success(scene.clone())),
            None => Ok(CommandResponse::error(&format!("Scene not found: {}", id))),
        },
        Err(e) => Ok(CommandResponse::error(&format!("No active project: {}", e))),
    }
}

#[tauri::command]
pub async fn activate_scene(
    id: String,
    state: State<'_, AppState>,
) -> Result<CommandResponse<bool>, String> {
    let mut project_manager = state.project_manager.lock().unwrap();

    match project_manager.activate_scene(&id) {
        Ok(_) => Ok(CommandResponse::success(true)),
        Err(e) => Ok(CommandResponse::error(&format!(
            "Failed to activate scene: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn assign_scene_to_grid(
    scene_id: String,
    position: u8,
    state: State<'_, AppState>,
) -> Result<CommandResponse<bool>, String> {
    let mut project_manager = state.project_manager.lock().unwrap();

    match project_manager.assign_scene_to_grid(&scene_id, position) {
        Ok(_) => Ok(CommandResponse::success(true)),
        Err(e) => Ok(CommandResponse::error(&format!(
            "Failed to assign scene to grid: {}",
            e
        ))),
    }
}

/// AI generation commands

#[tauri::command]
pub async fn generate_scene(
    params: GenerationParams,
    state: State<'_, AppState>,
) -> Result<CommandResponse<GeneratedScene>, String> {
    let scene_generator = state.scene_generator.lock().unwrap();

    match scene_generator.generate(params) {
        Ok(generated) => Ok(CommandResponse::success(generated)),
        Err(e) => Ok(CommandResponse::error(&format!(
            "Failed to generate scene: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn save_generated_scene(
    generated_scene: GeneratedScene,
    state: State<'_, AppState>,
) -> Result<CommandResponse<String>, String> {
    let mut project_manager = state.project_manager.lock().unwrap();

    // Get the active project
    match project_manager.get_active_project() {
        Ok(mut project) => {
            // Add the scene to the project
            let scene_id = generated_scene.scene.id.clone();
            project.add_scene(generated_scene.scene);

            // Save the project
            match project_manager.save_active_project() {
                Ok(_) => Ok(CommandResponse::success(scene_id)),
                Err(e) => Ok(CommandResponse::error(&format!(
                    "Failed to save scene: {}",
                    e
                ))),
            }
        }
        Err(e) => Ok(CommandResponse::error(&format!("No active project: {}", e))),
    }
}

#[tauri::command]
pub fn debug_connect_controller(deviceId: String) -> Result<String, String> {
    // Simple echo command to debug parameter passing - note the camelCase parameter name!
    Ok(format!("Received device ID: {}", deviceId))
}

#[tauri::command]
pub async fn check_backend_status() -> Result<String, String> {
    Ok(String::from("Backend is running and responsive"))
}

#[tauri::command]
pub async fn list_midi_devices(
    state: State<'_, AppState>,
) -> Result<CommandResponse<Vec<MidiDevice>>, String> {
    let project_manager = state.project_manager.lock().unwrap();

    // Get devices through the public method instead of accessing private field
    match ProjectManager::get_midi_devices() {
        Ok(devices) => Ok(CommandResponse::success(devices)),
        Err(e) => Ok(CommandResponse::error(&format!(
            "Failed to get MIDI devices: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn connect_controller(
    deviceId: String,
    state: State<'_, AppState>,
) -> Result<CommandResponse<bool>, String> {
    let mut project_manager = state.project_manager.lock().unwrap();

    // Convert camelCase parameter to snake_case for internal use
    match project_manager.connect_controller(&deviceId) {
        Ok(_) => Ok(CommandResponse::success(true)),
        Err(e) => Ok(CommandResponse::error(&format!(
            "Failed to connect controller: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn disconnect_controller(
    state: State<'_, AppState>,
) -> Result<CommandResponse<bool>, String> {
    let mut project_manager = state.project_manager.lock().unwrap();

    match project_manager.disconnect_controller() {
        Ok(_) => Ok(CommandResponse::success(true)),
        Err(e) => Ok(CommandResponse::error(&format!(
            "Failed to disconnect controller: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn send_cc(
    channel: u8,
    cc_number: u8,
    value: u8,
    state: State<'_, AppState>,
) -> Result<CommandResponse<bool>, String> {
    let project_manager = state.project_manager.lock().unwrap();

    match project_manager.send_cc(channel, cc_number, value) {
        Ok(_) => Ok(CommandResponse::success(true)),
        Err(e) => Ok(CommandResponse::error(&format!("Failed to send CC: {}", e))),
    }
}

#[command]
pub fn debug_midi_parameters(
    deviceId: String,
    isController: bool,
    otherParams: Option<String>,
) -> Result<String, String> {
    let mut response = format!(
        "Received parameters:\n- deviceId: {}\n- isController: {}",
        deviceId, isController
    );

    if let Some(params) = otherParams {
        response.push_str(&format!("\n- otherParams: {}", params));
    }

    Ok(response)
}
