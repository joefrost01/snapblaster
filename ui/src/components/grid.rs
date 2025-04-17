use crate::models::{Project, Scene, TriggerMode};
use leptos::prelude::*;
use web_sys::DragEvent;

#[component]
pub fn SceneGrid(
    project: Project,
    active_scene: ReadSignal<Option<Scene>>,
    on_activate: Callback<String>,
    on_assign: Callback<(String, u8)>,
) -> impl IntoView {
    let (dragging, set_drag) = create_signal(None::<String>);

    /* ---- pre‑compute cell info ---- */
    let cells = create_memo(move |_| {
        (0u8..64)
            .map(|pos| {
                let sid = project.grid_assignments.get(&pos.to_string()).cloned();
                let scene = sid.as_ref().and_then(|id| project.scenes.get(id)).cloned();
                let active = matches!((active_scene.get(), sid.as_ref()),
                                      (Some(a), Some(id)) if a.id == *id);
                (pos, scene, active)
            })
            .collect::<Vec<_>>()
    });

    /* ---- event helpers ---- */
    let drop_cb = move |pos: u8, e: DragEvent| {
        e.prevent_default();
        if let Some(id) = dragging.get() {
            on_assign.clone().run((id, pos));
        }
        set_drag.set(None);
    };

    let drag_cb = move |sid: String, e: DragEvent| {
        if let Some(dt) = e.data_transfer() {
            let _ = dt.set_data("text/plain", &sid);
        }
        set_drag.set(Some(sid));
    };

    /* ---- UI ---- */
    view! {
        <div class="scene-grid">
            <For
                each=move || cells.get()
                key=|(pos, _, _)| *pos
                children=move |(pos, scene_opt, active)| {
                    let style = scene_opt
                        .as_ref()
                        .and_then(|s| s.color.map(|(r, g, b)| format!("background-color:rgb({r},{g},{b});")))
                        .unwrap_or_default();

                    /* dynamic class */
                    let css_cls = {
                        let mut c = "scene-pad".to_string();
                        if scene_opt.is_none() { c.push_str(" empty"); }
                        if active    { c.push_str(" active"); }
                        if scene_opt.as_ref().map(|s| !matches!(s.trigger_mode, TriggerMode::Immediate)).unwrap_or(false) {
                            c.push_str(" transition");
                        }
                        c
                    };

                    // Clone scene ID for each closure separately
                    let scene_id_for_click = scene_opt.as_ref().map(|s| s.id.clone());
                    let scene_id_for_drag = scene_opt.as_ref().map(|s| s.id.clone());
                    let scene_name = scene_opt.as_ref().map(|s| s.name.clone()).unwrap_or_default();

                    let on_click = move |_| {
                        if let Some(id) = scene_id_for_click.clone() {
                            on_activate.clone().run(id);
                        }
                    };

                    let on_drag_start = move |e: DragEvent| {
                        if let Some(id) = scene_id_for_drag.clone() {
                            drag_cb(id, e);
                        }
                    };

                    let on_drop = move |e| drop_cb(pos, e);

                    view! {
                        <div class=css_cls
                             style=style
                             draggable="true"
                             on:dragstart=on_drag_start
                             on:dragover=move |e| e.prevent_default()
                             on:drop=on_drop
                             on:click=on_click>
                            <div class="scene-number">{(pos + 1).to_string()}</div>
                            <div class="scene-name">{scene_name}</div>
                        </div>
                    }
                }
            />
        </div>
    }
}
