use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;
use crate::tauri_commands::{check_backend_status, debug_connect_controller};

#[component]
pub fn DiagnosticPanel() -> impl IntoView {
    let (status, set_status) = create_signal("Unknown".to_string());
    let (is_checking, set_checking) = create_signal(false);
    let (device_id, set_device_id) = create_signal("".to_string());
    let (connect_result, set_connect_result) = create_signal("Not tested".to_string());

    let check_status = move |_| {
        set_checking.set(true);
        set_status.set("Checking...".to_string());

        spawn_local(async move {
            match check_backend_status().await {
                Ok(msg) => set_status.set(msg),
                Err(e) => set_status.set(format!("Error: {}", e)),
            }
            set_checking.set(false);
        });
    };

    let test_connect = move |_| {
        let id = device_id.get();
        if id.is_empty() {
            set_connect_result.set("Please enter a device ID".to_string());
            return;
        }

        set_connect_result.set("Testing...".to_string());

        spawn_local(async move {
            match debug_connect_controller(id).await {
                Ok(msg) => set_connect_result.set(msg),
                Err(e) => set_connect_result.set(format!("Error: {}", e)),
            }
        });
    };

    view! {
        <div class="diagnostic-panel">
            <h3>"Backend Diagnostics"</h3>
            <div class="status-display">
                <span class="label">"Status: "</span>
                <span class="value"
                      class:error=move || status.get().starts_with("Error")
                      class:checking=move || status.get() == "Checking..."
                      class:success=move || !status.get().starts_with("Error") && status.get() != "Checking...">
                    {move || status.get()}
                </span>
            </div>
            <button 
                on:click=check_status
                disabled=move || is_checking.get()>
                "Check Backend Status"
            </button>

            <div class="connect-test" style="margin-top: 1rem;">
                <h4>"Test MIDI Connection"</h4>
                <div style="display: flex; margin-bottom: 0.5rem;">
                    <input 
                        type="text" 
                        placeholder="Enter device ID"
                        on:input=move |e| {
                            let value = event_target::<web_sys::HtmlInputElement>(&e).value();
                            set_device_id.set(value);
                        }
                        style="flex: 1; margin-right: 0.5rem;"
                    />
                    <button on:click=test_connect>"Test"</button>
                </div>
                <div class="status-display">
                    <span class="label">"Result: "</span>
                    <span class="value">{move || connect_result.get()}</span>
                </div>
            </div>
        </div>
    }
}

fn event_target<T: wasm_bindgen::JsCast>(e: &leptos::ev::Event) -> T {
    e.target().unwrap().unchecked_into()
}