use crate::models::MidiDevice;
use leptos::prelude::*;
use leptos::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlSelectElement;

#[component]
pub fn MidiDeviceList(
    devices: ReadSignal<Vec<MidiDevice>>,
    on_connect: Callback<String>,
) -> impl IntoView {
    let (selected, set_selected) = create_signal(None::<String>);

    view! {
        <div class="midi-device-list">
            <div
                class="no-devices-message"
                style=move || if devices.get().is_empty() { "" } else { "display: none;" }
            >
                "No MIDI devices detected"
            </div>
            <div
                class="devices-container"
                style=move || if devices.get().is_empty() { "display: none;" } else { "" }
            >
                <select
                    on:change=move |e| {
                        let sel = event_target::<HtmlSelectElement>(&e).value();
                        set_selected.set(Some(sel));
                    }
                >
                    <option
                        value=""
                        disabled=true
                        selected=move || devices.get().is_empty()
                    >
                        "Select a MIDI device"
                    </option>
                    {devices.get()
                        .iter()
                        .filter(|d| d.is_controller)
                        .map(|d| view! {
                            <option value={d.id.clone()}>
                                {d.name.clone()} " (Controller)"
                            </option>
                        })
                        .collect::<Vec<_>>()}
                    {devices.get()
                        .iter()
                        .filter(|d| !d.is_input && !d.is_controller)
                        .map(|d| view! {
                            <option value={d.id.clone()}>
                                {d.name.clone()} " (Output)"
                            </option>
                        })
                        .collect::<Vec<_>>()}
                </select>

                <button
                    class="connect-button"
                    disabled=move || selected.get().is_none()
                    on:click=move |_| {
                        if let Some(dev) = selected.get() {
                            on_connect.clone().run(dev);
                        }
                    }
                >
                    "Connect"
                </button>
            </div>
        </div>
    }
}

fn event_target<T: JsCast>(e: &leptos::ev::Event) -> T {
    e.target().unwrap().unchecked_into::<T>()
}
