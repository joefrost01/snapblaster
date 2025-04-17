use crate::components::*;
use crate::models::{MidiDevice, Project, Scene};
use crate::tauri_commands::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;
use crate::console_log;

/* helper: DOM cast */
fn event_target<T: JsCast>(e: &leptos::ev::Event) -> T {
    e.target().unwrap().unchecked_into()
}

#[component]
pub fn App() -> impl IntoView {
    /* ---------- state ---------- */
    let (proj, set_proj) = create_signal(None::<Project>);
    let (scene, set_scene) = create_signal(None::<Scene>);
    let (devices, set_dev) = create_signal(Vec::<MidiDevice>::new());
    let (loading, set_load) = create_signal(false);
    let (error, set_err) = create_signal(None::<String>);

    let (show_proj, set_show_proj) = create_signal(false);
    let (show_set, set_show_set) = create_signal(false);
    let (show_ai, set_show_ai) = create_signal(false);

    /* ---------- side‑effects ---------- */
    // Setup a one-time MIDI device poll on startup
    spawn_local(async move {
        match list_midi_devices().await {
            Ok(d) => {
                console_log!("Found {} MIDI devices", d.len());
                set_dev.set(d);
            }
            Err(e) => {
                console_log!("MIDI device error: {}", e);
                set_err.set(Some(format!("MIDI error: {}", e)));
            }
        }
    });

    /* ---------- helpers ---------- */
    let load_project = move |id: String| {
        set_load.set(true);
        spawn_local(async move {
            match load_project_command(id).await {
                Ok(p) => {
                    set_proj.set(Some(p));
                    set_scene.set(None);
                }
                Err(e) => set_err.set(Some(e)),
            }
            set_load.set(false);
        });
    };

    let create_project = move |(name, author): (String, Option<String>)| {
        set_load.set(true);
        spawn_local(async move {
            match create_project_command(name, author).await {
                Ok(id) => load_project(id),
                Err(e) => {
                    set_err.set(Some(e));
                    set_load.set(false);
                }
            }
        });
    };

    let activate_scene = move |sid: String| {
        spawn_local(async move {
            if activate_scene_command(sid.clone()).await.is_ok() {
                if let Some(p) = proj.get() {
                    if let Some(s) = p.scenes.get(&sid) {
                        set_scene.set(Some(s.clone()));
                    }
                }
            }
        });
    };

    let assign_scene = move |(sid, pos): (String, u8)| {
        spawn_local(async move {
            match assign_scene_to_grid_command(sid, pos).await {
                Ok(_) => {
                    if let Some(p) = proj.get() {
                        load_project(p.id.clone());
                    }
                }
                Err(e) => set_err.set(Some(e)),
            }
        });
    };

    /* ---------- view ---------- */
    view! {
        <div class="app-container">
            /* ----- header ----- */
            <header class="app-header">
                <h1>"Snap‑Blaster"</h1>
                <button on:click=move |_| set_show_proj.set(true)>"Projects"</button>
                <button on:click=move |_| set_show_set.set(true) >"Settings"</button>
            </header>

            /* ----- body ----- */
            <main class="app-main">
                /* sidebar */
                <aside class="app-sidebar">
                    <h2>"Project"</h2>
                    <Show
                        when=move || proj.get().is_some()
                        fallback=move || view!{
                            <div class="project-placeholder">
                                <p>"No project loaded"</p>
                                <button on:click=move |_| set_show_proj.set(true)>"Open Project"</button>
                            </div>
                        }
                    >
                        {move || {
                            let p = proj.get().unwrap();
                            view!{
                                <div class="project-info">
                                    <h3>{p.name.clone()}</h3>
                                    <p>{p.description.clone().unwrap_or_default()}</p>
                                    <button on:click=move |_| set_show_ai.set(true)>"Create AI Scene"</button>
                                </div>
                            }
                        }}
                    </Show>

                    <h2>"MIDI Devices"</h2>
                    <midi_monitor::MidiDeviceList
                        devices=devices
                        on_connect=Callback::new(move |d| spawn_local(async move {
                            match connect_controller_command(d).await {
                                Ok(_) => console_log!("Connected to device successfully"),
                                Err(e) => console_log!("Error connecting: {}", e),
                            }
                        }))
                    />
                </aside>

                /* main area */
                <section class="app-content">
                    /* grid */
                    <div class="scene-grid-container">
                        <Show
                            when=move || proj.get().is_some()
                            fallback=move || view!{ <div class="grid-placeholder">"Please load a project"</div> }
                        >
                            {move || view!{
                                <grid::SceneGrid
                                    project=proj.get().unwrap()
                                    active_scene=scene
                                    on_activate=Callback::new(activate_scene.clone())
                                    on_assign=Callback::new(assign_scene.clone())
                                />
                            }}
                        </Show>
                    </div>

                    /* editor */
                    <div class="scene-editor">
                        <Show
                            when=move || scene.get().is_some()
                            fallback=move || view!{ <div class="editor-placeholder">"Select a scene to edit"</div> }
                        >
                            {move || view!{
                                <scene_editor::SceneEditor
                                    scene=scene.get().unwrap()
                                    cc_definitions=proj.get().map(|p| p.cc_definitions.clone()).unwrap_or_default()
                                    on_update=Callback::new(|_| {/* TODO */})
                                />
                            }}
                        </Show>
                    </div>
                </section>
            </main>

            /* dialogs */
            <Show when=move || show_proj.get() fallback=|| ().into_view()>
                {move || view!{
                    <dialogs::ProjectDialog
                        on_close=Callback::new(move |_| set_show_proj.set(false))
                        on_create=Callback::new(create_project.clone())
                        on_load=Callback::new(load_project.clone())
                    />
                }}
            </Show>

            <Show when=move || show_set.get() fallback=|| ().into_view()>
                {move || view!{
                    <dialogs::SettingsPanel
                        project=proj.get()
                        on_close=Callback::new(move |_| set_show_set.set(false))
                        _on_save=Callback::new(move |_p| set_show_set.set(false))
                    />
                }}
            </Show>

            <Show when=move || show_ai.get() fallback=|| ().into_view()>
                {move || view!{
                    <dialogs::AIPromptDialog
                        project=proj.get()
                        on_close=Callback::new(move |_| set_show_ai.set(false))
                        on_generate=Callback::new(move |_p| set_show_ai.set(false))
                    />
                }}
            </Show>

            <Show when=move || error.get().is_some() fallback=|| ().into_view()>
                {move || view!{
                    <dialogs::ErrorDialog
                        message=error.get().unwrap()
                        on_close=Callback::new(move |_| set_err.set(None))
                    />
                }}
            </Show>

            /* global loading */
            <div class="loading-overlay"
                 style=move || if loading.get() { "" } else { "display:none;" }>
                "Loading…"
            </div>
        </div>
    }
}