use leptos::*;
use leptos::prelude::{create_effect, create_signal, Callback, ClassAttribute, Get, GlobalAttributes, OnAttribute, Set};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;

use crate::models::ProjectMeta;
use crate::tauri_commands::list_projects;

#[component]
pub fn ProjectDialog(
    on_close: Callback<()>,
    on_create: Callback<(String, Option<String>)>,
    on_load: Callback<String>,
) -> impl IntoView {
    // Current tab
    let (active_tab, set_active_tab) = create_signal("existing");

    // Form fields for new project
    let (new_name, set_new_name) = create_signal(String::new());
    let (new_author, set_new_author) = create_signal(String::new());

    // Loading state
    let (is_loading, set_is_loading) = create_signal(false);
    let (projects, set_projects) = create_signal(Vec::<ProjectMeta>::new());
    let (selected_project, set_selected_project) = create_signal(None::<String>);

    // Load existing projects
    create_effect(move |_| {
        set_is_loading.set(true);

        spawn_local(async move {
            match list_projects().await {
                Ok(project_list) => set_projects.set(project_list),
                Err(_) => set_projects.set(Vec::new()),
            }

            set_is_loading.set(false);
        });
    });

    // Handle create new project
    let handle_create = move |_| {
        let name = new_name.get();
        let author = new_author.get();

        if name.trim().is_empty() {
            return;
        }

        let author_option = if author.trim().is_empty() {
            None
        } else {
            Some(author)
        };

        on_create.call((name, author_option));
        on_close.call(());
    };

    // Handle load existing project
    let handle_load = move |_| {
        if let Some(id) = selected_project.get() {
            on_load.call(id);
            on_close.call(());
        }
    };

    view! {
        <div class="dialog-overlay">
            <div class="dialog project-dialog">
                <div class="dialog-header">
                    <h2>"Projects"</h2>
                    <button 
                        class="close-button"
                        on:click=move |_| on_close.call(())
                    >
                        "×"
                    </button>
                </div>
                
                <div class="dialog-tabs">
                    <button 
                        class=move || if active_tab.get() == "existing" { "tab active" } else { "tab" }
                        on:click=move |_| set_active_tab.set("existing")
                    >
                        "Existing Projects"
                    </button>
                    <button 
                        class=move || if active_tab.get() == "new" { "tab active" } else { "tab" }
                        on:click=move |_| set_active_tab.set("new")
                    >
                        "New Project"
                    </button>
                </div>
                
                <div class="dialog-content">
                    {move || match active_tab.get().as_str() {
                        "existing" => view! {
                            <div class="tab-content existing-projects">
                                {move || if is_loading.get() {
                                    view! { <div class="loading">"Loading projects..."</div> }
                                } else if projects.get().is_empty() {
                                    view! { <div class="no-projects">"No projects found"</div> }
                                } else {
                                    view! {
                                        <div class="project-list">
                                            {projects.get().into_iter().map(|project| {
                                                let id = project.id.clone();
                                                view! {
                                                    <div 
                                                        class=move || {
                                                            if Some(id.clone()) == selected_project.get() {
                                                                "project-item"
                                                            }
                                                        }
                                                        on:click=move |_| set_selected_project.set(Some(id.clone()))
                                                    >
                                                        <div class="project-name">{project.name}</div>
                                                        <div class="project-details">
                                                            {project.description.unwrap_or_default()}
                                                            {move || project.author.map(|author| view! {
                                                                <span class="project-author">{" by "}{author}</span>
                                                            })}
                                                        </div>
                                                        <div class="project-date">
                                                            {"Last updated: "}
                                                            {format_date(&project.updated_at)}
                                                        </div>
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }
                                }}
                            </div>
                        },
                        "new" => view! {
                            <div class="tab-content new-project">
                                <div class="form-group">
                                    <label for="project-name">"Project Name"</label>
                                    <input 
                                        type="text"
                                        id="project-name"
                                        placeholder="Enter project name"
                                        value=new_name
                                        on:input=move |e| set_new_name.set(event_target_value(&e))
                                    />
                                </div>
                                
                                <div class="form-group">
                                    <label for="project-author">"Author (optional)"</label>
                                    <input 
                                        type="text"
                                        id="project-author"
                                        placeholder="Enter author name"
                                        value=new_author
                                        on:input=move |e| set_new_author.set(event_target_value(&e))
                                    />
                                </div>
                                
                                <div class="template-options">
                                    <h3>"Template Options"</h3>
                                    <div class="templates">
                                        <div class="template-item selected">
                                            <div class="template-name">"Default Project"</div>
                                            <div class="template-description">
                                                "Basic project with common MIDI CC mappings"
                                            </div>
                                        </div>
                                        <div class="template-item disabled">
                                            <div class="template-name">"Techno Template"</div>
                                            <div class="template-description">
                                                "Optimized for techno performance (Pro version)"
                                            </div>
                                        </div>
                                        <div class="template-item disabled">
                                            <div class="template-name">"Ambient Template"</div>
                                            <div class="template-description">
                                                "Smooth transitions for ambient music (Pro version)"
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        },
                        _ => view! { <div>"Invalid tab"</div> }
                    }}
                </div>
                
                <div class="dialog-footer">
                    {move || match active_tab.get().as_str() {
                        "existing" => view! {
                            <button 
                                class="button secondary"
                                on:click=move |_| on_close.call(())
                            >
                                "Cancel"
                            </button>
                            <button 
                                class="button primary"
                                disabled=move || selected_project.get().is_none()
                                on:click=handle_load
                            >
                                "Load Project"
                            </button>
                        },
                        "new" => view! {
                            <button 
                                class="button secondary"
                                on:click=move |_| on_close.call(())
                            >
                                "Cancel"
                            </button>
                            <button 
                                class="button primary"
                                disabled=move || new_name.get().trim().is_empty()
                                on:click=handle_create
                            >
                                "Create Project"
                            </button>
                        },
                        _ => view! {}
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

// Helper function to format date strings
fn format_date(date_str: &str) -> String {
    // Simple format, in a real app you might use a date library
    let parts: Vec<&str> = date_str.split('T').collect();
    if parts.len() > 1 {
        parts[0].to_string()
    } else {
        date_str.to_string()
    }
}