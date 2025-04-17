use leptos::prelude::mount_to_body;
use leptos::*;
use wasm_bindgen::prelude::*;

mod app;
mod components;
mod models;
mod tauri_commands;


// Import for macro use
#[macro_use]
pub mod macros {
    #[macro_export]
    macro_rules! console_log {
        ($($arg:tt)*) => {
            web_sys::console::log_1(&format!($($arg)*).into());
        }
    }
}

use app::App;

fn main() {
    console_error_panic_hook::set_once();
    console_log!("Starting Snap-Blaster UI...");

    mount_to_body(|| {
        view! {
            <App />
        }
    });
}