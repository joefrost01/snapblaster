use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, Runtime};

use crate::ai::generator::SceneGenerator;
use crate::commands::AppState;
use crate::link::integration::LinkIntegration;
use crate::midi::devices::DeviceRegistryFactory;
use crate::midi::engine::MidiEngine;
use crate::project::manager::ProjectManager;
use crate::project::storage::ProjectStorage;

/// Initialize the application
pub fn init<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    // Initialize project storage
    let storage = ProjectStorage::init()
        .map_err(|e| format!("Failed to initialize project storage: {}", e))?;

    // Initialize device registry
    let device_registry = Arc::new(
        DeviceRegistryFactory::create()
            .map_err(|e| format!("Failed to initialize device registry: {}", e))?,
    );

    // Initialize MIDI engine
    let midi_engine =
        Arc::new(Mutex::new(MidiEngine::new().map_err(|e| {
            format!("Failed to initialize MIDI engine: {}", e)
        })?));

    // Start the MIDI engine
    midi_engine
        .lock()
        .unwrap()
        .start()
        .map_err(|e| format!("Failed to start MIDI engine: {}", e))?;

    // Initialize project manager
    let project_manager = Arc::new(Mutex::new(ProjectManager::new(
        storage,
        Arc::clone(&device_registry),
        Arc::clone(&midi_engine),
    )));

    // Initialize AI scene generator
    let api_key = std::env::var("OPENAI_API_KEY").ok();
    let scene_generator = Arc::new(Mutex::new(SceneGenerator::new(api_key)));

    // Initialize Link integration
    let default_tempo = 120.0; // Default tempo
    let mut link = LinkIntegration::new(default_tempo);

    // Set up Link callbacks
    let midi_engine_for_link = Arc::clone(&midi_engine);
    link.set_beat_callback(move |beat| {
        // Handle beat events
        if let Ok(mut engine) = midi_engine_for_link.lock() {
            // Update engine with beat information
            let _ = engine.send_command(crate::midi::engine::MidiCommand::SetTempo(default_tempo));
        }
    });

    // Create app state
    let state = AppState {
        project_manager: Arc::clone(&project_manager),
        scene_generator: Arc::clone(&scene_generator),
    };

    // Store state in the app
    app.manage(state);

    // Set up event handlers for window events
    let app_handle = app.clone();
    let midi_engine_clone = Arc::clone(&midi_engine);

    app.listen_global("tauri://close-requested", move |_| {
        // Shut down the MIDI engine
        if let Ok(mut engine) = midi_engine_clone.lock() {
            let _ = engine.shutdown();
        }

        // Exit the application
        app_handle.exit(0);
    });

    Ok(())
}

/// Run background scan for MIDI devices
pub fn scan_midi_devices_background<R: Runtime>(app: &AppHandle<R>) {
    let app_handle = app.clone();

    // Start a thread to scan for MIDI devices periodically
    std::thread::spawn(move || {
        loop {
            // Get the state
            if let Some(state) = app_handle.try_state::<AppState>() {
                let mut project_manager = state.project_manager.lock().unwrap();

                // Scan for devices
                let _ = project_manager.device_registry.scan_devices();

                // Emit an event to notify the frontend
                let _ = app_handle.emit_all("devices-updated", ());
            }

            // Sleep for 2 seconds before scanning again
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    });
}
