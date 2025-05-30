use tauri::{Error, Runtime};

mod ai;
mod app;
mod commands;
mod link;
mod midi;
mod models;
mod project;

#[cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Initialize the application
            app::init(&app.handle()).map_err(|e| {
                // Convert the string error to a tauri::Error
                Error::from(std::io::Error::new(std::io::ErrorKind::Other, e))
            })?;

            // Start background scan for MIDI devices
            app::scan_midi_devices_background(&app.handle());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Project commands
            commands::list_projects,
            commands::get_active_project,
            commands::create_project,
            commands::create_template_project,
            commands::load_project,
            commands::save_project,
            commands::import_project,
            commands::export_project,
            commands::close_project,
            // Scene commands
            commands::create_scene,
            commands::get_scene,
            commands::activate_scene,
            commands::assign_scene_to_grid,
            // MIDI device commands
            commands::list_midi_devices,
            commands::connect_controller,
            commands::disconnect_controller,
            commands::send_cc,
            // AI generation commands
            commands::generate_scene,
            commands::save_generated_scene,
            // Diagnostic commands
            commands::check_backend_status,
            commands::debug_connect_controller,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}