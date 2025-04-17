use leptos::*;
use std::collections::HashMap;
use leptos::prelude::{create_memo, create_signal, Callback, Get, Set};
use leptos::task::spawn_local;
use web_sys::HtmlInputElement;

use crate::models::{Project, GenerationParams, CCDefinitionRef, SceneRef, Scene, GeneratedScene};
use crate::tauri_commands::{generate_scene_command, save_generated_scene_command};

#[component]
pub fn AIPromptDialog(
    #[prop(optional)]
    project: Option<Project>,
    on_close: Callback<()>,
    on_generate: Callback<GenerationParams>,
) -> impl IntoView {
    // Form state
    let (description, set_description) = create_signal(String::new());
    let (use_transitions, set_use_transitions) = create_signal(true);
    let (add_randomness, set_add_randomness) = create_signal(false);
    let (selected_tags, set_selected_tags) = create_signal(Vec::<String>::new());
    let (reference_scene, set_reference_scene) = create_signal(None::<String>);

    // Processing state
    let (is_generating, set_is_generating) = create_signal(false);
    let (generated_scene, set_generated_scene) = create_signal(None::<GeneratedScene>);
    let (error_message, set_error_message) = create_signal(None::<String>);

    // Available tags from all scenes
    let available_tags = create_memo(move |_| {
        let mut tags = Vec::new();

        if let Some(project) = project.clone() {
            for scene in project.scenes.values() {
                for tag in &scene.tags {
                    if !tags.contains(tag) {
                        tags.push(tag.clone());
                    }
                }
            }
        }

        tags.sort();
        tags
    });

    // Handle toggling a tag
    let toggle_tag = move |tag: String| {
        let mut current = selected_tags.get();

        if current.contains(&tag) {
            current.retain(|t| t != &tag);
        } else {
            current.push(tag);
        }

        set_selected_tags.set(current);
    };

    // Handle generating a scene
    let handle_generate = move |_| {
        set_is_generating.set(true);
        set_error_message.set(None);

        // Only proceed if we have a project
        let project_clone = match project.clone() {
            Some(p) => p,
            None => {
                set_error_message.set(Some("No active project".to_string()));
                set_is_generating.set(false);
                return;
            }
        };

        // Build generation parameters
        let mut params = GenerationParams {
            description: description.get(),
            cc_definitions: Vec::new(),
            previous_scene: None,
            use_transitions: use_transitions.get(),
            add_randomness: add_randomness.get(),
            tags: selected_tags.get(),
        };

        // Add CC definitions from project
        for cc_def in project_clone.cc_definitions.values() {
            params.cc_definitions.push(CCDefinitionRef {
                channel: cc_def.channel,
                cc_number: cc_def.cc_number,
                name: cc_def.name.clone(),
                description: cc_def.description.clone(),
            });
        }

        // Add reference scene if selected
        if let Some(scene_id) = reference_scene.get() {
            if let Some(scene) = project_clone.scenes.get(&scene_id) {
                params.previous_scene = Some(SceneRef {
                    id: scene.id.clone(),
                    name: scene.name.clone(),
                    cc_values: scene.cc_values.clone(),
                });
            }
        }

        // Clone parameters for callback
        let params_clone = params.clone();

        // Call the generate function
        spawn_local(async move {
            match generate_scene_command(params).await {
                Ok(scene) => {
                    set_generated_scene.set(Some(scene));
                    set_is_generating.set(false);
                },
                Err(e) => {
                    set_error_message.set(Some(format!("Failed to generate scene: {}", e)));
                    set_is_generating.set(false);
                }
            }
        });

        // Call the callback
        on_generate(params_clone);
    };

    // Handle saving the generated scene
    let handle_save = move |_| {
        if let Some(generated) = generated_scene.get() {
            spawn_local(async move {
                match save_generated_scene_command(generated).await {
                    Ok(_) => {
                        // Close the dialog after saving
                        on_close(());
                    },
                    Err(e) => {
                        set_error_message.set(Some(format!("Failed to save scene: {}", e)));
                    }
                }
            });
        }
    };

    view! {
        <div class="dialog-overlay">
            <div class="dialog ai-prompt-dialog">
                <div class="dialog-header">
                    <h2>"AI Scene Generator"</h2>
                    <button
                        class="close-button"
                        on:click=move |_| on_close(())
                    >
                        "×"
                    </button>
                </div>

                <div class="dialog-content">
                    {move || if generated_scene.get().is_some() {
                        // Show the generated scene
                        let scene = generated_scene.get().unwrap();

                        view! {
                            <div class="generated-scene">
                                <h3>{"Generated Scene: "}{scene.scene.name.clone()}</h3>

                                <div class="scene-info">
                                    <div class="form-group">
                                        <label>"Description"</label>
                                        <p>{scene.scene.description.clone().unwrap_or_default()}</p>
                                    </div>

                                    <div class="form-group">
                                        <label>"AI Explanation"</label>
                                        <p class="explanation">{scene.explanation.clone()}</p>
                                    </div>

                                    <div class="form-group">
                                        <label>"CC Values"</label>
                                        <div class="cc-value-list">
                                            {scene.scene.cc_values.values().take(5).map(|cc| {
                                                let name = cc.name.clone().unwrap_or_else(|| format!("CC {}", cc.cc_number));
                                                view! {
                                                    <div class="cc-value-item">
                                                        <span class="cc-name">{name}</span>
                                                        <span class="cc-value">{cc.value}</span>
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>()}

                                            {move || if scene.scene.cc_values.len() > 5 {
                                                let remaining = scene.scene.cc_values.len() - 5;
                                                view! { <div class="more-cc-values">{"... and "}{remaining}{" more"}</div> }
                                            } else {
                                                view! {}
                                            }}
                                        </div>
                                    </div>
                                </div>
                            </div>
                        }
                    } else {
                        // Show the prompt form
                        view! {
                            <div class="ai-prompt-form">
                                <div class="form-group">
                                    <label for="scene-description">"Describe the scene you want"</label>
                                    <textarea
                                        id="scene-description"
                                        rows="4"
                                        placeholder="Example: A dark, filtered intro that slowly opens up with increasing resonance"
                                        value=description
                                        on:input=move |e| {
                                            let target = event_target::<web_sys::HtmlTextAreaElement>(&e);
                                            set_description.set(target.value());
                                        }
                                    ></textarea>
                                </div>

                                <div class="form-group">
                                    <label>"Reference Scene (optional)"</label>
                                    <select
                                        on:change=move |e| {
                                            let target = event_target::<web_sys::HtmlSelectElement>(&e);
                                            let value = target.value();
                                            set_reference_scene.set(if value.is_empty() { None } else { Some(value) });
                                        }
                                    >
                                        <option value="">"None"</option>
                                        {move || if let Some(p) = project.clone() {
                                            p.scenes.values().map(|scene| {
                                                view! {
                                                    <option value={scene.id.clone()}>{scene.name.clone()}</option>
                                                }
                                            }).collect::<Vec<_>>()
                                        } else {
                                            vec![]
                                        }}
                                    </select>
                                </div>

                                <div class="form-group">
                                    <label>"Tags (optional)"</label>
                                    <div class="tags-container">
                                        {move || available_tags.get().into_iter().map(|tag| {
                                            let tag_clone = tag.clone();
                                            let is_selected = selected_tags.with(|tags| tags.contains(&tag));

                                            view! {
                                                <div
                                                    class=move || if is_selected() { "tag selected" } else { "tag" }
                                                    on:click=move |_| toggle_tag(tag_clone.clone())
                                                >
                                                    {tag_clone}
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                </div>

                                <div class="form-group options">
                                    <div class="option-item">
                                        <label class="checkbox-label">
                                            <input
                                                type="checkbox"
                                                checked=use_transitions
                                                on:change=move |e| {
                                                    let target = event_target::<web_sys::HtmlInputElement>(&e);
                                                    set_use_transitions.set(target.checked());
                                                }
                                            />
                                            "Use transitions"
                                        </label>
                                    </div>

                                    <div class="option-item">
                                        <label class="checkbox-label">
                                            <input
                                                type="checkbox"
                                                checked=add_randomness
                                                on:change=move |e| {
                                                    let target = event_target::<web_sys::HtmlInputElement>(&e);
                                                    set_add_randomness.set(target.checked());
                                                }
                                            />
                                            "Add some randomness"
                                        </label>
                                    </div>
                                </div>

                                {move || if let Some(error) = error_message.get() {
                                    view! { <div class="error-message">{error}</div> }
                                } else {
                                    view! {}
                                }}
                            </div>
                        }
                    }}
                </div>

                <div class="dialog-footer">
                    <button
                        class="button secondary"
                        on:click=move |_| on_close(())
                    >
                        "Cancel"
                    </button>

                    {move || if generated_scene.get().is_some() {
                        view! {
                            <>
                                <button
                                    class="button secondary"
                                    on:click=move |_| set_generated_scene.set(None)
                                >
                                    "Generate Another"
                                </button>
                                <button
                                    class="button primary"
                                    on:click=handle_save
                                >
                                    "Save Scene"
                                </button>
                            </>
                        }
                    } else {
                        view! {
                            <button
                                class="button primary"
                                disabled=move || description.get().trim().is_empty() || is_generating.get()
                                on:click=handle_generate
                            >
                                {move || if is_generating.get() { "Generating..." } else { "Generate Scene" }}
                            </button>
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

// Helper function to get the event target
fn event_target<T: wasm_bindgen::JsCast>(event: &leptos::ev::Event) -> T {
    event.target().unwrap().unchecked_into::<T>()
}