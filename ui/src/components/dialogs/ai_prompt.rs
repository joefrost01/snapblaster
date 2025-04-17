use leptos::html::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;

use crate::models::{CCDefinitionRef, GeneratedScene, GenerationParams, Project, SceneRef};
use crate::tauri_commands::{generate_scene_command, save_generated_scene_command};

/* handy DOM‑cast helper */
fn event_target<T: JsCast>(e: &leptos::ev::Event) -> T {
    e.target().unwrap().unchecked_into()
}

/* “… and N more” — single‑path, no branching */
#[component]
fn MoreCcValues(count: usize) -> impl IntoView {
    let txt = if count > 5 {
        format!("… and {} more", count - 5)
    } else {
        String::new()
    };
    view! { <div class="more-cc-values">{txt}</div> }
}

#[component]
pub fn AIPromptDialog(
    project: Option<Project>,
    on_close: Callback<()>,
    on_generate: Callback<GenerationParams>,
) -> impl IntoView {
    /* ---------- local state ---------- */
    let (description, set_desc) = create_signal(String::new());
    let (use_trans, set_ut) = create_signal(true);
    let (add_rand, set_rand) = create_signal(false);
    let (tags, set_tags) = create_signal(Vec::<String>::new());
    let (reference, set_ref) = create_signal(None::<String>);

    let (is_gen, set_igen) = create_signal(false);
    let (generated, set_gen) = create_signal(None::<GeneratedScene>);
    let (err_msg, set_err) = create_signal(None::<String>);

    /* ---------- derived ---------- */
    let project_signal = create_rw_signal(project);

    let all_tags = create_memo(move |_| {
        let mut t = Vec::<String>::new();
        if let Some(p) = project_signal.get() {
            for scene in p.scenes.values() {
                for tag in &scene.tags {
                    if !t.contains(tag) {
                        t.push(tag.clone());
                    }
                }
            }
        }
        t.sort();
        t
    });

    /* ---------- helpers ---------- */
    let toggle_tag = move |t: String| {
        let mut cur = tags.get();
        if !cur.iter().any(|x| x == &t) {
            cur.push(t);
        } else {
            cur.retain(|x| x != &t);
        }
        set_tags.set(cur);
    };

    /* ---------- generate ---------- */
    let gen_click = move |_| {
        set_igen.set(true);
        set_err.set(None);

        let proj = match project_signal.get() {
            Some(p) => p.clone(),
            None => {
                set_err.set(Some("No active project".into()));
                set_igen.set(false);
                return;
            }
        };

        let mut params = GenerationParams {
            description: description.get(),
            cc_definitions: proj
                .cc_definitions
                .values()
                .map(|d| CCDefinitionRef {
                    channel: d.channel,
                    cc_number: d.cc_number,
                    name: d.name.clone(),
                    description: d.description.clone(),
                })
                .collect(),
            previous_scene: None,
            use_transitions: use_trans.get(),
            add_randomness: add_rand.get(),
            tags: tags.get(),
        };

        if let Some(id) = reference.get() {
            if let Some(s) = proj.scenes.get(&id) {
                params.previous_scene = Some(SceneRef {
                    id: s.id.clone(),
                    name: s.name.clone(),
                    cc_values: s.cc_values.clone(),
                });
            }
        }

        on_generate.run(params.clone());
        spawn_local(async move {
            match generate_scene_command(params).await {
                Ok(sc) => set_gen.set(Some(sc)),
                Err(e) => set_err.set(Some(format!("Failed: {e}"))),
            }
            set_igen.set(false);
        });
    };

    /* ---------- save ---------- */
    let save_click = move |_| {
        if let Some(gs) = generated.get() {
            spawn_local(async move {
                match save_generated_scene_command(gs).await {
                    Ok(_) => on_close.clone().run(()),
                    Err(e) => set_err.set(Some(format!("Save failed: {e}"))),
                }
            });
        }
    };

    /* ---------- UI pieces ---------- */
    let prompt_form = move || {
        view! {
            <div class="ai-prompt-form">
                /* description */
                <div class="form-group">
                    <label>"Describe the scene"</label>
                    <textarea rows="4"
                        on:input=move |e| set_desc.set(
                            event_target::<web_sys::HtmlTextAreaElement>(&e).value()
                        )/>
                </div>

                /* reference scene */
                <div class="form-group">
                    <label>"Reference Scene"</label>
                    <select on:change=move |e| {
                        let v = event_target::<web_sys::HtmlSelectElement>(&e).value();
                        set_ref.set(if v.is_empty() { None } else { Some(v) });
                    }>
                        <option value="">"None"</option>
                        {move || project_signal
                            .get()
                            .map(|p| p.scenes.values().map(|s| {
                                view! { <option value={s.id.clone()}>{s.name.clone()}</option> }
                            }).collect::<Vec<_>>())
                            .unwrap_or_default()}
                    </select>
                </div>

                /* tags */
                <div class="form-group">
                    <label>"Tags"</label>
                    <div class="tags-container">
                        {move || all_tags.get().into_iter().map(|tg| {
                            let label    = tg.clone();                 // keep a copy for rendering
                            let selected = tags.with(|ts| ts.contains(&tg));
                            view! {
                                <div class=move || if selected { "tag selected" } else { "tag" }
                                     on:click=move |_| toggle_tag(tg.clone())>
                                    {label}
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>

                /* options */
                <div class="form-group options">
                    <label>
                        <input type="checkbox"
                               checked=use_trans
                               on:change=move |e| set_ut.set(
                                   event_target::<web_sys::HtmlInputElement>(&e).checked()
                               )/>
                        "Use transitions"
                    </label>
                    <label>
                        <input type="checkbox"
                               checked=add_rand
                               on:change=move |e| set_rand.set(
                                   event_target::<web_sys::HtmlInputElement>(&e).checked()
                               )/>
                        "Add randomness"
                    </label>
                </div>

                /* inline error */
                <Show
                    when=move || err_msg.get().is_some()
                    fallback=|| view!{ <div class="error-message" style="display:none;" /> }
                >
                    {move || view!{ <div class="error-message">{err_msg.get().unwrap()}</div> }}
                </Show>
            </div>
        }
    };

    /* called only when `generated` is Some – no branching inside */
    let generated_preview = move || {
        let sc = generated.get().unwrap(); // safety: guarded by `<Show>`
        let name = sc.scene.name.clone();
        let description = sc.scene.description.clone().unwrap_or_default();
        let explanation = sc.explanation.clone();
        let cc_values = sc.scene.cc_values.clone();
        let count = cc_values.len();

        view! {
            <>
                <h3>{"Generated: "}{name}</h3>
                <p>{description}</p>

                <div class="cc-value-list">
                    {cc_values.values().take(5).map(|cc| {
                        let nm = cc.name.clone().unwrap_or_else(|| format!("CC {}", cc.cc_number));
                        view! {
                            <div class="cc-value-item">
                                <span class="cc-name">{nm}</span>
                                <span class="cc-value">{cc.value}</span>
                            </div>
                        }
                    }).collect::<Vec<_>>()}

                    <MoreCcValues count={count}/>
                </div>

                <p class="explanation">{explanation}</p>
            </>
        }
    };

    /* ---------- top‑level dialog ---------- */
    view! {
        <div class="dialog-overlay">
            <div class="dialog ai-prompt-dialog">

                /* header */
                <div class="dialog-header">
                    <h2>"AI Scene Generator"</h2>
                    <button class="close-button"
                            on:click=move |_| on_close.clone().run(())>
                        "×"
                    </button>
                </div>

                /* content */
                <div class="dialog-content">
                    <Show when=move || generated.get().is_some()
                          fallback=prompt_form >
                        {generated_preview}
                    </Show>
                </div>

                /* footer */
                <div class="dialog-footer">
                    <button class="button secondary"
                            on:click=move |_| on_close.clone().run(())>
                        "Cancel"
                    </button>

                    <Show when=move || generated.get().is_some()
                          fallback=move || {
                              let disabled =
                                  description.with(|d| d.trim().is_empty()) || is_gen.get();
                              view! {
                                  <button class="button primary"
                                          disabled=move || disabled
                                          on:click=gen_click>
                                      {move || if is_gen.get() { "Generating…" } else { "Generate Scene" }}
                                  </button>
                              }
                          }>
                        {view! {
                            <>
                                <button class="button secondary"
                                        on:click=move |_| set_gen.set(None)>
                                    "Generate Another"
                                </button>
                                <button class="button primary"
                                        on:click=save_click>
                                    "Save Scene"
                                </button>
                            </>
                        }}
                    </Show>
                </div>
            </div>
        </div>
    }
}
