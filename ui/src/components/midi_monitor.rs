use leptos::*;
use leptos::prelude::{create_signal, Callback, ClassAttribute, Get, OnAttribute, ReadSignal, Set};
use crate::models::MidiDevice;

#[component]
pub fn MidiDeviceList(
    devices: ReadSignal<Vec<MidiDevice>>,
    on_connect: Callback<String>,
) -> impl IntoView {
    // Track which device is selected
    let (selected_device, set_selected_device) = create_signal(None::<String>);

    view! {
        <div class="midi-device-list">
            {move || {
                let device_list = devices.get();

                if device_list.is_empty() {
                    view! { <p class="no-devices-message">"No MIDI devices detected"</p> }
                } else {
                    view! {
                        <div>
                            <select
                                on:change=move |e| {
                                    let target = event_target::<web_sys::HtmlSelectElement>(&e);
                                    set_selected_device.set(Some(target.value()));
                                }
                            >
                                <option value="" selected=true disabled=true>"Select a MIDI device"</option>

                                // Filter to show only controller devices
                                {device_list.iter().filter(|d| d.is_controller).map(|device| {
                                    view! {
                                        <option value={device.id.clone()}>
                                            {device.name.clone()} " (Controller)"
                                        </option>
                                    }
                                }).collect::<Vec<_>>()}

                                // Then show output devices
                                {device_list.iter().filter(|d| !d.is_input && !d.is_controller).map(|device| {
                                    view! {
                                        <option value={device.id.clone()}>
                                            {device.name.clone()} " (Output)"
                                        </option>
                                    }
                                }).collect::<Vec<_>>()}
                            </select>

                            <button
                                class="connect-button"
                                disabled=move || selected_device.get().is_none()
                                on:click=move |_| {
                                    if let Some(device_id) = selected_device.get() {
                                        on_connect.call(device_id);
                                    }
                                }
                            >
                                "Connect"
                            </button>
                        </div>
                    }
                }
            }}
        </div>
    }
}

// Helper function to get the event target
fn event_target<T: wasm_bindgen::JsCast>(event: &leptos::ev::Event) -> T {
    event.target().unwrap().unchecked_into::<T>()
}