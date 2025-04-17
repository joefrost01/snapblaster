use leptos::*;
use std::collections::HashMap;
use leptos::prelude::{create_memo, create_signal, Callback, Get, Set};
use wasm_bindgen::JsCast;
use crate::models::{Scene, CCValue, CCDefinition, TriggerMode};
use crate::components::CCEditor;

#[component]
pub fn SceneEditor(
    scene: Scene,
    cc_definitions: HashMap<String, CCDefinition>,
    on_update: Callback<Scene>,
) -> impl IntoView {
    // Create local state for editing
    let (name, set_name) = create_signal(scene.name.clone());
    let (description, set_description) = create_signal(scene.description.clone().unwrap_or_default());
    let (trigger_mode, set_trigger_mode) = create_signal(scene.trigger_mode.clone());
    let (cc_values, set_cc_values) = create_signal(scene.cc_values.clone());
    let (is_editing, set_is_editing) = create_signal(false);
    let (is_dirty, set_is_dirty) = create_signal(false);

    // String representation of trigger mode for the UI
    let trigger_mode_string = create_memo(move |_| {
        match trigger_mode.get() {
            TriggerMode::Immediate => "Immediate".to_string(),
            TriggerMode::NextBeat => "Next Beat".to_string(),
            TriggerMode::Beats(n) => format!("{} Beats", n),
            TriggerMode::NextBar => "Next Bar".to_string(),
        }
    });

    // Group CC values by channel
    let grouped_cc_values = create_memo(move |_| {
        let values = cc_values.get();
        let mut grouped = HashMap::new();

        for (key, value) in values.iter() {
            let parts: Vec<&str> = key.split(':').collect();
            if parts.len() == 2 {
                if let (Ok(channel), Ok(cc_number)) = (parts[0].parse::<u8>(), parts[1].parse::<u8>()) {
                    let group_key = format!("Channel {}", channel);
                    let entry = grouped.entry(group_key).or_insert_with(Vec::new);
                    entry.push((cc_number, value.clone()));
                }
            }
        }

        // Sort each group by CC number
        for values in grouped.values_mut() {
            values.sort_by_key(|(num, _)| *num);
        }

        grouped
    });

    // Handle updating a CC value
    let update_cc_value = move |channel: u8, cc_number: u8, new_value: CCValue| {
        let key = format!("{}:{}", channel, cc_number);
        let mut updated = cc_values.get();
        updated.insert(key, new_value);
        set_cc_values.set(updated);
        set_is_dirty.set(true);
    };

    // Handle saving changes
    let save_changes = move |_| {
        let mut updated_scene = scene.clone();
        updated_scene.name = name.get();
        updated_scene.description = Some(description.get()).filter(|s| !s.is_empty());
        updated_scene.trigger_mode = trigger_mode.get();
        updated_scene.cc_values = cc_values.get();

        on_update.call(updated_scene);
        set_is_editing.set(false);
        set_is_dirty.set(false);
    };

    view! {
        <div class="scene-editor-container">
            <div class="scene-editor-header">
                {move || if is_editing.get() {
                    view! {
                        <div class="edit-header">
                            <input 
                                type="text"
                                value=name
                                on:input=move |e| {
                                    set_name.set(event_target_value(&e));
                                    set_is_dirty.set(true);
                                }
                                placeholder="Scene Name"
                            />
                            <div class="edit-actions">
                                <button 
                                    class="save-button" 
                                    on:click=save_changes
                                    disabled=move || !is_dirty.get()
                                >
                                    "Save"
                                </button>
                                <button 
                                    class="cancel-button" 
                                    on:click=move |_| {
                                        set_is_editing.set(false);
                                        // Reset to original values
                                        set_name.set(scene.name.clone());
                                        set_description.set(scene.description.clone().unwrap_or_default());
                                        set_trigger_mode.set(scene.trigger_mode.clone());
                                        set_cc_values.set(scene.cc_values.clone());
                                        set_is_dirty.set(false);
                                    }
                                >
                                    "Cancel"
                                </button>
                            </div>
                        </div>
                    }
                } else {
                    view! {
                        <div class="view-header">
                            <h2>{name}</h2>
                            <button 
                                class="edit-button" 
                                on:click=move |_| set_is_editing.set(true)
                            >
                                "Edit"
                            </button>
                        </div>
                    }
                }}
            </div>
            
            <div class="scene-editor-details">
                <div class="scene-meta">
                    <div class="detail-row">
                        <span class="label">"Trigger Mode:"</span>
                        <span class="value">{trigger_mode_string}</span>
                    </div>
                    
                    {move || if !description.get().is_empty() {
                        view! {
                            <div class="detail-row">
                                <span class="label">"Description:"</span>
                                <span class="value">{description}</span>
                            </div>
                        }
                    } else {
                        view! {}
                    }}
                </div>
                
                <div class="cc-values-container">
                    <h3>"CC Values"</h3>
                    
                    {move || {
                        let groups = grouped_cc_values.get();
                        
                        if groups.is_empty() {
                            view! { <p class="empty-message">"No CC values in this scene"</p> }
                        } else {
                            groups.into_iter().map(|(group_name, values)| {
                                view! {
                                    <div class="cc-group">
                                        <h4>{group_name}</h4>
                                        <div class="cc-list">
                                            {values.into_iter().map(|(cc_number, value)| {
                                                let cc_def = cc_definitions.values()
                                                    .find(|def| def.channel == value.channel && def.cc_number == cc_number)
                                                    .cloned();
                                                
                                                view! {
                                                    <CCEditor
                                                        value=value.clone()
                                                        definition=cc_def
                                                        is_editing=is_editing
                                                        on_change=move |new_value| {
                                                            update_cc_value(value.channel, cc_number, new_value);
                                                        }
                                                    />
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    </div>
                                }
                            }).collect::<Vec<_>>()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

// Helper function to get input value
fn event_target_value(event: &leptos::ev::Event) -> String {
    event_target::<web_sys::HtmlInputElement>(event).value()
}

// Helper function to get the event target
fn event_target<T: wasm_bindgen::JsCast>(event: &leptos::ev::Event) -> T {
    event.target().unwrap().unchecked_into::<T>()
}