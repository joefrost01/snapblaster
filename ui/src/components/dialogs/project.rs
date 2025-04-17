use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;
use web_sys::MouseEvent;

use crate::models::ProjectMeta;
use crate::tauri_commands::list_projects;

/* tiny helpers */
fn val(e: &leptos::ev::Event) -> String {
    event_target::<web_sys::HtmlInputElement>(e).value()
}
fn event_target<T: JsCast>(e: &leptos::ev::Event) -> T {
    e.target().unwrap().unchecked_into()
}
fn format_date(raw: &str) -> String {
    raw.split('T').next().unwrap_or(raw).into()
}

#[component]
pub fn ProjectDialog(
    on_close: Callback<()>,
    on_create: Callback<(String, Option<String>)>,
    on_load: Callback<String>,
) -> impl IntoView {
    /* -------- state -------- */
    let (active_tab, set_tab) = create_signal("existing".to_string());
    let (new_name, set_new_name) = create_signal(String::new());
    let (new_author, set_author) = create_signal(String::new());

    let (is_loading, set_loading) = create_signal(false);
    let (projects, set_projects) = create_signal(Vec::<ProjectMeta>::new());
    let (selected, set_selected) = create_signal(None::<String>);

    /* ---- load once on mount ---- */
    create_effect(move |_| {
        set_loading.set(true);
        spawn_local(async move {
            match list_projects().await {
                Ok(p) => set_projects.set(p),
                Err(_) => set_projects.set(Vec::new()),
            }
            set_loading.set(false);
        });
    });

    /* -------- callbacks -------- */
    let handle_create = move |_| {
        if new_name.get().trim().is_empty() {
            return;
        }
        let author = if new_author.get().trim().is_empty() {
            None
        } else {
            Some(new_author.get())
        };
        on_create.run((new_name.get(), author));
        on_close.run(());
    };

    let handle_load = move |_| {
        if let Some(id) = selected.get() {
            on_load.run(id);
            on_close.run(());
        }
    };

    /* ------ helper sub‑views ------ */
    let existing_view = move || {
        view! {
            <Show
                when=move || !is_loading.get()
                fallback=move || view!{ <div class="loading">"Loading projects…"</div> }
            >
                <Show
                    when=move || !projects.get().is_empty()
                    fallback=move || view!{ <div class="no-projects">"No projects found"</div> }
                >
                    {move || view!{
                        <div class="project-list">
                            {projects.get().iter().map(|proj| {
                                // Clone outside the closures to avoid move issues
                                let id_for_selection = proj.id.clone();
                                let id_for_click = proj.id.clone();
                                let is_sel = move || Some(id_for_selection.clone()) == selected.get();

                                view!{
                                    <div class=move || if is_sel() { "project-item selected" } else { "project-item" }
                                         on:click=move |_| set_selected.set(Some(id_for_click.clone()))>
                                        <div class="project-name">{proj.name.clone()}</div>
                                        <div class="project-details">
                                            {proj.description.clone().unwrap_or_default()}
                                            {proj.author.clone().map(|a| view!{
                                                <span class="project-author">{" by "}{a}</span>
                                            }).into_view()}
                                        </div>
                                        <div class="project-date">
                                            {"Last updated: "}{format_date(&proj.updated_at)}
                                        </div>
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }}
                </Show>
            </Show>
        }
    };

    let new_view = move || {
        view! {
            <div class="new-project tab-content">
                <div class="form-group">
                    <label for="project-name">"Project Name"</label>
                    <input id="project-name" type="text"
                           placeholder="Enter name"
                           prop:value=move || new_name.get()
                           on:input=move |e| set_new_name.set(val(&e))/>
                </div>
                <div class="form-group">
                    <label for="project-author">"Author (optional)"</label>
                    <input id="project-author" type="text"
                           prop:value=move || new_author.get()
                           on:input=move |e| set_author.set(val(&e))/>
                </div>
            </div>
        }
    };

    /* --------------- top‑level view --------------- */
    view! {
        <div class="dialog-overlay">
            <div class="dialog project-dialog">
                /* ----- header ----- */
                <div class="dialog-header">
                    <h2>"Projects"</h2>
                    <button class="close-button"
                            on:click=move |_| on_close.clone().run(())>
                        "×"
                    </button>
                </div>

                /* ----- tab bar ----- */
                <div class="dialog-tabs">
                    <button
                        class=move || if active_tab.get() == "existing" { "tab active" } else { "tab" }
                        on:click=move |_| set_tab.set("existing".into())>
                        "Existing"
                    </button>
                    <button
                        class=move || if active_tab.get() == "new" { "tab active" } else { "tab" }
                        on:click=move |_| set_tab.set("new".into())>
                        "New"
                    </button>
                </div>

                /* ----- body ----- */
                <div class="dialog-content">
                    <Show
                        when=move || active_tab.get() == "existing"
                        fallback=new_view
                    >
                        {existing_view}
                    </Show>
                </div>

                /* ----- footer ----- */
                <div class="dialog-footer">
                    <button class="button secondary"
                            on:click=move |_| on_close.clone().run(())>
                        "Cancel"
                    </button>

                    {move || {
                        let is_existing = active_tab.get() == "existing";
                        let disabled = move || {
                            if is_existing {
                                selected.get().is_none()
                            } else {
                                new_name.get().trim().is_empty()
                            }
                        };
                        let onclick = move |e: MouseEvent| {
                            e.stop_propagation();
                            if is_existing { handle_load(e) } else { handle_create(e) }
                        };
                        let label = if is_existing { "Load Project" } else { "Create Project" };
                        view!{
                            <button class="button primary"
                                    disabled=disabled
                                    on:click=onclick>
                                {label}
                            </button>
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
