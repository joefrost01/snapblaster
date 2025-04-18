use crate::models::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use crate::console_log;

// Helper function to invoke Tauri commands
pub(crate) async fn invoke<T, R>(command: &str, args: Option<T>) -> Result<R, String>
where
    T: Serialize,
    R: for<'de> Deserialize<'de>,
{
    let window = web_sys::window().unwrap();

    // Access the `__TAURI__` object
    let tauri = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__"))
        .map_err(|_| "Tauri API not found".to_string())?;

    // Access the `invoke` function
    let invoke_fn = js_sys::Reflect::get(&tauri, &JsValue::from_str("invoke"))
        .map_err(|_| "Tauri invoke function not found".to_string())?;

    // Create the arguments
    let args_value = match args {
        Some(data) => {
            let serialized = serde_wasm_bindgen::to_value(&data)
                .map_err(|e| format!("Failed to serialize args: {:?}", e))?;

            // Debug the serialized value
            let json_string = js_sys::JSON::stringify(&serialized)
                .map_err(|_| "Failed to stringify arguments".to_string())?
                .as_string()
                .unwrap_or_default();

            console_log!("Invoking {} with args: {}", command, json_string);

            serialized
        },
        None => JsValue::NULL,
    };

    // Call the invoke function
    let promise = js_sys::Reflect::apply(
        &invoke_fn.dyn_into::<js_sys::Function>().unwrap(),
        &tauri,
        &js_sys::Array::of3(
            &JsValue::from_str(command),
            &args_value,
            &JsValue::NULL, // No options
        ),
    )
        .map_err(|e| format!("Failed to call Tauri invoke: {:?}", e))?;

    // Wait for the promise to resolve
    let result = JsFuture::from(promise.dyn_into::<js_sys::Promise>().unwrap())
        .await
        .map_err(|e| format!("Tauri command failed: {:?}", e))?;

    // Deserialize the result
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to deserialize result: {:?}", e))
}

// Project management commands

pub async fn list_projects() -> Result<Vec<ProjectMeta>, String> {
    invoke("list_projects", None::<()>).await
}

pub async fn get_active_project() -> Result<Project, String> {
    let response: CommandResponse<Project> = invoke("get_active_project", None::<()>).await?;

    match response {
        CommandResponse {
            success: true,
            data: Some(project),
            ..
        } => Ok(project),
        CommandResponse {
            success: false,
            error: Some(err),
            ..
        } => Err(err),
        _ => Err("Unknown error getting active project".to_string()),
    }
}

pub async fn create_project_command(
    name: String,
    author: Option<String>,
) -> Result<String, String> {
    #[derive(Serialize)]
    struct CreateProjectArgs {
        name: String,
        author: Option<String>,
    }

    let args = CreateProjectArgs { name, author };
    let response: CommandResponse<String> = invoke("create_project", Some(args)).await?;

    match response {
        CommandResponse {
            success: true,
            data: Some(id),
            ..
        } => Ok(id),
        CommandResponse {
            success: false,
            error: Some(err),
            ..
        } => Err(err),
        _ => Err("Unknown error creating project".to_string()),
    }
}

pub async fn load_project_command(id: String) -> Result<Project, String> {
    #[derive(Serialize)]
    struct LoadProjectArgs {
        id: String,
    }

    let args = LoadProjectArgs { id };
    let response: CommandResponse<Project> = invoke("load_project", Some(args)).await?;

    match response {
        CommandResponse {
            success: true,
            data: Some(project),
            ..
        } => Ok(project),
        CommandResponse {
            success: false,
            error: Some(err),
            ..
        } => Err(err),
        _ => Err("Unknown error loading project".to_string()),
    }
}

pub async fn save_project() -> Result<bool, String> {
    let response: CommandResponse<bool> = invoke("save_project", None::<()>).await?;

    match response {
        CommandResponse {
            success: true,
            data: Some(saved),
            ..
        } => Ok(saved),
        CommandResponse {
            success: false,
            error: Some(err),
            ..
        } => Err(err),
        _ => Err("Unknown error saving project".to_string()),
    }
}

// Scene management commands

pub async fn create_scene_command(
    name: String,
    description: Option<String>,
) -> Result<String, String> {
    #[derive(Serialize)]
    struct CreateSceneArgs {
        name: String,
        description: Option<String>,
    }

    let args = CreateSceneArgs { name, description };
    let response: CommandResponse<String> = invoke("create_scene", Some(args)).await?;

    match response {
        CommandResponse {
            success: true,
            data: Some(id),
            ..
        } => Ok(id),
        CommandResponse {
            success: false,
            error: Some(err),
            ..
        } => Err(err),
        _ => Err("Unknown error creating scene".to_string()),
    }
}

pub async fn get_scene_command(id: String) -> Result<Scene, String> {
    #[derive(Serialize)]
    struct GetSceneArgs {
        id: String,
    }

    let args = GetSceneArgs { id };
    let response: CommandResponse<Scene> = invoke("get_scene", Some(args)).await?;

    match response {
        CommandResponse {
            success: true,
            data: Some(scene),
            ..
        } => Ok(scene),
        CommandResponse {
            success: false,
            error: Some(err),
            ..
        } => Err(err),
        _ => Err("Unknown error getting scene".to_string()),
    }
}

pub async fn activate_scene_command(id: String) -> Result<bool, String> {
    #[derive(Serialize)]
    struct ActivateSceneArgs {
        id: String,
    }

    let args = ActivateSceneArgs { id };
    let response: CommandResponse<bool> = invoke("activate_scene", Some(args)).await?;

    match response {
        CommandResponse {
            success: true,
            data: Some(activated),
            ..
        } => Ok(activated),
        CommandResponse {
            success: false,
            error: Some(err),
            ..
        } => Err(err),
        _ => Err("Unknown error activating scene".to_string()),
    }
}

pub async fn assign_scene_to_grid_command(scene_id: String, position: u8) -> Result<bool, String> {
    #[derive(Serialize)]
    struct AssignSceneArgs {
        scene_id: String,
        position: u8,
    }

    let args = AssignSceneArgs { scene_id, position };
    let response: CommandResponse<bool> = invoke("assign_scene_to_grid", Some(args)).await?;

    match response {
        CommandResponse {
            success: true,
            data: Some(assigned),
            ..
        } => Ok(assigned),
        CommandResponse {
            success: false,
            error: Some(err),
            ..
        } => Err(err),
        _ => Err("Unknown error assigning scene to grid".to_string()),
    }
}

// MIDI device commands

pub async fn list_midi_devices() -> Result<Vec<MidiDevice>, String> {
    let response: CommandResponse<Vec<MidiDevice>> =
        invoke("list_midi_devices", None::<()>).await?;

    match response {
        CommandResponse {
            success: true,
            data: Some(devices),
            ..
        } => Ok(devices),
        CommandResponse {
            success: false,
            error: Some(err),
            ..
        } => Err(err),
        _ => Err("Unknown error listing MIDI devices".to_string()),
    }
}

pub async fn disconnect_controller_command() -> Result<bool, String> {
    let response: CommandResponse<bool> = invoke("disconnect_controller", None::<()>).await?;

    match response {
        CommandResponse {
            success: true,
            data: Some(disconnected),
            ..
        } => Ok(disconnected),
        CommandResponse {
            success: false,
            error: Some(err),
            ..
        } => Err(err),
        _ => Err("Unknown error disconnecting controller".to_string()),
    }
}

pub async fn send_cc_command(channel: u8, cc_number: u8, value: u8) -> Result<bool, String> {
    #[derive(Serialize)]
    struct SendCCArgs {
        channel: u8,
        cc_number: u8,
        value: u8,
    }

    let args = SendCCArgs {
        channel,
        cc_number,
        value,
    };
    let response: CommandResponse<bool> = invoke("send_cc", Some(args)).await?;

    match response {
        CommandResponse {
            success: true,
            data: Some(sent),
            ..
        } => Ok(sent),
        CommandResponse {
            success: false,
            error: Some(err),
            ..
        } => Err(err),
        _ => Err("Unknown error sending CC".to_string()),
    }
}

// AI generation commands

pub async fn generate_scene_command(params: GenerationParams) -> Result<GeneratedScene, String> {
    let response: CommandResponse<GeneratedScene> = invoke("generate_scene", Some(params)).await?;

    match response {
        CommandResponse {
            success: true,
            data: Some(generated),
            ..
        } => Ok(generated),
        CommandResponse {
            success: false,
            error: Some(err),
            ..
        } => Err(err),
        _ => Err("Unknown error generating scene".to_string()),
    }
}

pub async fn save_generated_scene_command(
    generated_scene: GeneratedScene,
) -> Result<String, String> {
    let response: CommandResponse<String> =
        invoke("save_generated_scene", Some(generated_scene)).await?;

    match response {
        CommandResponse {
            success: true,
            data: Some(id),
            ..
        } => Ok(id),
        CommandResponse {
            success: false,
            error: Some(err),
            ..
        } => Err(err),
        _ => Err("Unknown error saving generated scene".to_string()),
    }
}

// Debug helper to test backend connectivity
pub async fn check_backend_status() -> Result<String, String> {
    match invoke::<(), String>("check_backend_status", None).await {
        Ok(msg) => Ok(msg),
        Err(e) => Err(format!("Backend connectivity error: {}", e)),
    }
}

// Debug helper for connecting to a controller
pub async fn debug_connect_controller(device_id: String) -> Result<String, String> {
    // Use a HashMap to ensure parameter names are exactly as expected
    use std::collections::HashMap;

    let mut args = HashMap::new();
    args.insert("deviceId".to_string(), device_id.clone());

    console_log!("Debug connecting with HashMap: deviceId = {}", device_id);

    match invoke::<HashMap<String, String>, String>("debug_connect_controller", Some(args)).await {
        Ok(msg) => Ok(msg),
        Err(e) => Err(format!("Debug connect error: {}", e)),
    }
}

// More detailed MIDI parameter debugging
pub async fn debug_midi_parameters(device_id: String, is_controller: bool) -> Result<String, String> {
    #[derive(Serialize)]
    struct DebugMidiArgs {
        // All camelCase parameters to match the backend expectation
        deviceId: String,
        isController: bool,
        otherParams: Option<String>,
    }

    let args = DebugMidiArgs {
        deviceId: device_id,
        isController: is_controller,
        otherParams: Some("This is a test parameter".to_string()),
    };
    console_log!("Debug MIDI params: {:?}", serde_json::to_string(&args).unwrap_or_default());

    match invoke::<DebugMidiArgs, String>("debug_midi_parameters", Some(args)).await {
        Ok(msg) => Ok(msg),
        Err(e) => Err(format!("Debug MIDI params error: {}", e)),
    }
}

// Generic echo test for any parameters
pub async fn echo_params<T: Serialize>(params: T) -> Result<String, String> {
    match invoke::<T, String>("echo_params", Some(params)).await {
        Ok(msg) => Ok(msg),
        Err(e) => Err(format!("Echo params error: {}", e)),
    }
}


pub async fn connect_controller_command(device_id: String) -> Result<bool, String> {
    // Use a HashMap to ensure parameter names are exactly as expected
    use std::collections::HashMap;

    let mut args = HashMap::new();
    args.insert("deviceId".to_string(), device_id.clone());

    console_log!("Connecting with HashMap: deviceId = {}", device_id);

    let response: CommandResponse<bool> = invoke("connect_controller", Some(args)).await?;

    match response {
        CommandResponse {
            success: true,
            data: Some(connected),
            ..
        } => Ok(connected),
        CommandResponse {
            success: false,
            error: Some(err),
            ..
        } => Err(err),
        _ => Err("Unknown error connecting controller".to_string()),
    }
}

// A variant of invoke that accepts a JsValue directly
async fn invoke_raw<R>(command: &str, args: Option<js_sys::Object>) -> Result<R, String>
where
    R: for<'de> Deserialize<'de>,
{
    let window = web_sys::window().unwrap();

    // Access the `__TAURI__` object
    let tauri = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__"))
        .map_err(|_| "Tauri API not found".to_string())?;

    // Access the `invoke` function
    let invoke_fn = js_sys::Reflect::get(&tauri, &JsValue::from_str("invoke"))
        .map_err(|_| "Tauri invoke function not found".to_string())?;

    // Create the arguments
    let args_value = match args {
        Some(obj) => {
            // Debug the serialized value
            let json_string = js_sys::JSON::stringify(&obj)
                .map_err(|_| "Failed to stringify arguments".to_string())?
                .as_string()
                .unwrap_or_default();

            console_log!("Invoking {} with raw args: {}", command, json_string);
            JsValue::from(obj)
        },
        None => JsValue::NULL,
    };

    // Call the invoke function
    let promise = js_sys::Reflect::apply(
        &invoke_fn.dyn_into::<js_sys::Function>().unwrap(),
        &tauri,
        &js_sys::Array::of3(
            &JsValue::from_str(command),
            &args_value,
            &JsValue::NULL, // No options
        ),
    )
        .map_err(|e| format!("Failed to call Tauri invoke: {:?}", e))?;

    // Wait for the promise to resolve
    let result = JsFuture::from(promise.dyn_into::<js_sys::Promise>().unwrap())
        .await
        .map_err(|e| format!("Tauri command failed: {:?}", e))?;

    // Deserialize the result
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to deserialize result: {:?}", e))
}

