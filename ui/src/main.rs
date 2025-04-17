use leptos::*;
use leptos::prelude::mount_to_body;
use leptos_meta::*;
use wasm_bindgen::prelude::*;

mod components;
mod tauri_commands;
mod models;
mod app;

use components::*;
use app::App;

fn main() {
    console_error_panic_hook::set_once();

    mount_to_body(|| {
        view! {
            // Add meta information
            <Stylesheet id="main-stylesheet" href="/src/styles/main.css"/>
            <Stylesheet id="grid-stylesheet" href="/src/styles/grid.css"/>
            <Stylesheet id="editor-stylesheet" href="/src/styles/editor.css"/>
            <Stylesheet id="dialogs-stylesheet" href="/src/styles/dialogs.css"/>
            
            <App />
        }
    });
}