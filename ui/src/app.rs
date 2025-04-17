use leptos::*;
use leptos::prelude::{create_effect, create_signal, ClassAttribute, Get, OnAttribute, Set};
use leptos_meta::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::components::*;
use crate::models;
use crate::tauri_commands::*;
use crate::models::*;

#[component]
pub fn App() -> impl IntoView {
    // Application state
    let (active_project, set_active_project) = create_signal(None::<models::Project>);
    let (active_scene, set_active_scene) = create_signal(None::<models::Scene>);
    let (midi_devices, set_midi_devices) = create_signal(Vec::<models::MidiDevice>::new());
    let (is_loading, set_is_loading) = create_signal(false);
    let (error_message, set_error_message) = create_signal(None::<String>);
    let (show_project_dialog, set_show_project_dialog) = create_signal(false);
    let (show_settings_dialog, set_show_settings_dialog) = create_signal(false);
    let (show_ai_prompt_dialog, set_show_ai_prompt_dialog) = create_signal(false);

    // Load MIDI devices on component mount
    create_effect(move |_| {
        spawn_local(async move {
            match list_midi_devices().await {
                Ok(devices) => set_midi_devices.set(devices),
                Err(err) => set_error_message.set(Some(err))
            }
        });
    });

    // Project actions
    let load_project = move |id: String| {
        set_is_loading.set(true);
        spawn_local(async move {
            match load_project_command(id).await {
                Ok(project) => {
                    set_active_project.set(Some(project));
                    set_active_scene.set(None);
                }
                Err(err) => set_error_message.set(Some(err))
            }
            set_is_loading.set(false);
        });
    };

    let create_new_project = move |name: String, author: Option<String>| {
        set_is_loading.set(true);
        spawn_local(async move {
            match create_project_command(name, author).await {
                Ok(id) => load_project(id),
                Err(err) => {
                    set_error_message.set(Some(err));
                    set_is_loading.set(false);
                }
            }
        });
    };

    let activate_scene = move |id: String| {
        spawn_local(async move {
            match activate_scene_command(id.clone()).await {
                Ok(_) => {
                    if let Some(project) = active_project.get() {
                        if let Some(scene) = project.scenes.get(&id) {
                            set_active_scene.set(Some(scene.clone()));
                        }
                    }
                }
                Err(err) => set_error_message.set(Some(err))
            }
        });
    };

    let assign_to_grid = move |scene_id: String, position: u8| {
        spawn_local(async move {
            match assign_scene_to_grid_command(scene_id, position).await {
                Ok(_) => {
                    // Refresh project to get updated grid assignments
                    if let Some(project) = active_project.get() {
                        load_project(project.id.clone());
                    }
                }
                Err(err) => set_error_message.set(Some(err))
            }
        });
    };

    view! {
        <div class="app-container">
            <header class="app-header">
                <div class="logo">
                    <h1>Snap-Blaster</h1>
                </div>
                <div class="header-controls">
                    <button on:click=move |_| set_show_project_dialog.set(true)>
                        "Projects"
                    </button>
                    <button on:click=move |_| set_show_settings_dialog.set(true)>
                        "Settings"
                    </button>
                </div>
            </header>
            <main class="app-main">
                <aside class="app-sidebar">
                    <div class="sidebar-section">
                        <h2>Project</h2>
                        {move || match active_project.get() {
                            Some(project) => view! {
                                <div class="project-info">
                                    <h3>{project.name}</h3>
                                    <p>{project.description.unwrap_or_default()}</p>
                                    <button on:click=move |_| set_show_ai_prompt_dialog.set(true)>
                                        "Create AI Scene"
                                    </button>
                                </div>
                            },
                            None => view! {
                                <div class="project-placeholder">
                                    <p>"No project loaded"</p>
                                    <button on:click=move |_| set_show_project_dialog.set(true)>
                                        "Open Project"
                                    </button>
                                </div>
                            }
                        }}
                    </div>
                    <div class="sidebar-section">
                        <h2>MIDI Devices</h2>
                        <MidiDeviceList
                            devices=midi_devices
                            on_connect=move |device_id| {
                                spawn_local(async move {
                                    let _ = connect_controller_command(device_id).await;
                                });
                            }
                        />
                    </div>
                </aside>
                <div class="app-content">
                    <div class="scene-grid-container">
                        {move || match active_project.get() {
                            Some(project) => view! {
                                <SceneGrid
                                    project=project
                                    active_scene=active_scene
                                    on_activate=activate_scene
                                    on_assign=assign_to_grid
                                />
                            },
                            None => view! { <div class="no-project-message">"Please load a project"</div> }
                        }}
                    </div>
                    <div class="scene-editor">
                        {move || match active_scene.get() {
                            Some(scene) => view! {
                                <SceneEditor
                                    scene=scene
                                    cc_definitions=active_project.get().map(|p| p.cc_definitions).unwrap_or_default()
                                    on_update=move |updated| {
                                        // Handle scene updates
                                        // This would update the scene and save it
                                        // Simplified for now
                                    }
                                />
                            },
                            None => view! { <div class="no-scene-message">"Select a scene to edit"</div> }
                        }}
                    </div>
                </div>
            </main>

            // Dialogs
            {move || if show_project_dialog.get() {
                view! {
                    <dialogs::ProjectDialog
                        on_close=move || set_show_project_dialog.set(false)
                        on_create=create_new_project
                        on_load=load_project
                    />
                }
            } else {
                view! {}
            }}

            {move || if show_settings_dialog.get() {
                view! {
                    <dialogs::SettingsDialog
                        on_close=move || set_show_settings_dialog.set(false)
                        project=active_project.get()
                        on_save=move |_updated| {
                            // Handle settings update
                            set_show_settings_dialog.set(false);
                        }
                    />
                }
            } else {
                view! {}
            }}

            {move || if show_ai_prompt_dialog.get() {
                view! {
                    <dialogs::AIPromptDialog
                        on_close=move || set_show_ai_prompt_dialog.set(false)
                        project=active_project.get()
                        on_generate=move |_params| {
                            // Handle AI scene generation
                            set_show_ai_prompt_dialog.set(false);
                        }
                    />
                }
            } else {
                view! {}
            }}

            {move || if let Some(message) = error_message.get() {
                view! {
                    <dialogs::ErrorDialog
                        message=message
                        on_close=move || set_error_message.set(None)
                    />
                }
            } else {
                view! {}
            }}

            {move || if is_loading.get() {
                view! { <div class="loading-overlay">"Loading..."</div> }
            } else {
                view! {}
            }}
        </div>
    }
}