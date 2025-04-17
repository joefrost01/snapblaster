use crate::models::MidiDevice;
use crate::tauri_commands::{connect_controller_command, debug_connect_controller};
use leptos::prelude::*;
use leptos::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;
use web_sys::HtmlSelectElement;
use crate::console_log;

#[component]
pub fn MidiDeviceList(
    devices: ReadSignal<Vec<MidiDevice>>,
    on_connect: Callback<String>,
) -> impl IntoView {
    let (selected, set_selected) = create_signal(None::<String>);
    let (selected_name, set_selected_name) = create_signal("None".to_string());
    let (debug_result, set_debug_result) = create_signal("".to_string());

    let try_debug_connect = move |_| {
        if let Some(device_id) = selected.get() {
            set_debug_result.set("Testing connection...".to_string());

            spawn_local(async move {
                match debug_connect_controller(device_id.clone()).await {
                    Ok(result) => {
                        set_debug_result.set(format!("Debug: {}", result));

                        // If debug was successful, try the actual connect
                        match connect_controller_command(device_id.clone()).await {
                            Ok(_) => {
                                set_debug_result.set(format!("Connected successfully! Debug: {}", result));
                                on_connect.run(device_id);
                            },
                            Err(e) => {
                                set_debug_result.set(format!("Debug OK but real connect failed: {} | Debug: {}", e, result));
                            }
                        }
                    },
                    Err(e) => set_debug_result.set(format!("Debug error: {}", e)),
                }
            });
        } else {
            set_debug_result.set("Please select a device first".to_string());
        }
    };

    let try_real_connect = move |_| {
        if let Some(device_id) = selected.get() {
            set_debug_result.set("Connecting...".to_string());

            // First try with debug to see what's happening
            spawn_local(async move {
                match debug_connect_controller(device_id.clone()).await {
                    Ok(debug_result) => {
                        // Now try the actual connection
                        match connect_controller_command(device_id.clone()).await {
                            Ok(_) => {
                                set_debug_result.set(format!("Connected successfully! Debug: {}", debug_result));
                                on_connect.run(device_id);
                            },
                            Err(e) => set_debug_result.set(format!("Debug OK but real connect failed: {} | Debug: {}", e, debug_result)),
                        }
                    },
                    Err(e) => set_debug_result.set(format!("Debug error: {}", e)),
                }
            });
        } else {
            set_debug_result.set("Please select a device first".to_string());
        }
    };

    let try_direct_connect = move |_| {
        if let Some(device_id) = selected.get() {
            set_debug_result.set("Direct connecting...".to_string());

            // Use std::collections::HashMap directly without struct indirection
            spawn_local(async move {
                use std::collections::HashMap;

                // Create a simple hashmap with exactly the expected parameter name
                let mut params = HashMap::new();
                params.insert("deviceId".to_string(), device_id.clone());

                // Log what we're sending
                console_log!("Trying direct connection with deviceId = {}", device_id);

                // Call the invoke function from tauri_commands directly 
                match crate::tauri_commands::invoke::<HashMap<String, String>, crate::models::CommandResponse<bool>>(
                    "connect_controller",
                    Some(params)
                ).await {
                    Ok(response) => {
                        match response {
                            crate::models::CommandResponse { success: true, data: Some(_), .. } => {
                                set_debug_result.set("Direct connection successful!".to_string());
                                on_connect.run(device_id);
                            },
                            crate::models::CommandResponse { success: false, error: Some(e), .. } => {
                                set_debug_result.set(format!("Direct connection failed: {}", e));
                            },
                            _ => set_debug_result.set("Unknown response from connect command".to_string()),
                        }
                    },
                    Err(e) => set_debug_result.set(format!("Direct connection error: {}", e)),
                }
            });
        } else {
            set_debug_result.set("Please select a device first".to_string());
        }
    };

    view! {
        <div class="midi-device-list">
            <div
                class="no-devices-message"
                style=move || if devices.with(|d| d.is_empty()) { "" } else { "display: none;" }
            >
                "No MIDI devices detected"
            </div>
            <div
                class="devices-container"
                style=move || if devices.with(|d| d.is_empty()) { "display: none;" } else { "" }
            >
                <select
                    on:change=move |e| {
                        let sel = event_target::<HtmlSelectElement>(&e).value();
                        let name = event_target::<HtmlSelectElement>(&e).selected_options().item(0)
                                      .map(|o| o.text_content().unwrap_or_default())
                                      .unwrap_or_default();
                        
                        set_selected_name.set(name);
                        set_selected.set(if sel.is_empty() { None } else { Some(sel) });
                    }
                >
                    <option
                        value=""
                        disabled=true
                        selected=move || selected.get().is_none()
                    >
                        "Select a MIDI device"
                    </option>
                    
                    {move || {
                        devices.get().iter()
                            .filter(|d| d.is_controller)
                            .map(|d| {
                                let id = d.id.clone();
                                let name = d.name.clone();
                                view! {
                                    <option value={id}>
                                        {name} " (Controller)"
                                    </option>
                                }
                            })
                            .collect::<Vec<_>>()
                    }}
                    
                    {move || {
                        devices.get().iter()
                            .filter(|d| !d.is_input && !d.is_controller)
                            .map(|d| {
                                let id = d.id.clone();
                                let name = d.name.clone();
                                view! {
                                    <option value={id}>
                                        {name} " (Output)"
                                    </option>
                                }
                            })
                            .collect::<Vec<_>>()
                    }}
                </select>

                <div style="margin-top: 0.5rem; font-size: 0.8rem;">
                    <div>"Selected: " {move || selected_name.get()}</div>
                    <div style="margin-top: 0.25rem; word-break: break-all;">
                        "ID: " {move || selected.get().unwrap_or_default()}
                    </div>
                </div>

                <div style="display: flex; gap: 0.5rem; margin-top: 0.5rem;">
                    <button
                        class="connect-button"
                        disabled=move || selected.get().is_none()
                        on:click=move |_| {
                            if let Some(dev) = selected.get() {
                                on_connect.clone().run(dev);
                            }
                        }
                    >
                        "Connect (Original)"
                    </button>
                    
                    <button
                        class="debug-button"
                        disabled=move || selected.get().is_none()
                        on:click=try_debug_connect
                    >
                        "Debug Connect"
                    </button>
                </div>
                
                <div style="margin-top: 0.5rem; color: #ff9800; font-size: 0.9rem;">
                    {move || debug_result.get()}
                </div>
            </div>
        </div>
    }
}

fn event_target<T: JsCast>(e: &leptos::ev::Event) -> T {
    e.target().unwrap().unchecked_into::<T>()
}