use leptos::*;
use leptos::prelude::{create_memo, create_signal, Callback, ClassAttribute, Get, GlobalAttributes, OnAttribute, ReadSignal, Set, StyleAttribute};
use web_sys::DragEvent;
use wasm_bindgen::JsCast;

use crate::models::{Project, Scene};

#[component]
pub fn SceneGrid(
    project: Project,
    active_scene: ReadSignal<Option<Scene>>,
    on_activate: Callback<String>,
    on_assign: Callback<(String, u8)>,
) -> impl IntoView {
    // Track drag and drop state
    let (dragging_scene_id, set_dragging_scene_id) = create_signal(None::<String>);

    // Generate the grid of cells (8x8)
    let grid = create_memo(move |_| {
        let mut cells = Vec::with_capacity(64);

        for row in 0..8 {
            for col in 0..8 {
                let position = row * 8 + col;

                // Find if there's a scene assigned to this position
                let assigned_scene_id = project.grid_assignments.get(&position.to_string());
                let scene = assigned_scene_id.and_then(|id| project.scenes.get(id));

                // Check if this scene is active
                let is_active = match (active_scene.get(), assigned_scene_id) {
                    (Some(active), Some(assigned)) => active.id == *assigned,
                    _ => false
                };

                cells.push((position as u8, scene, is_active));
            }
        }

        cells
    });

    // Handle dropping a scene onto a grid cell
    let handle_drop = move |position: u8, event: DragEvent| {
        event.prevent_default();

        if let Some(scene_id) = dragging_scene_id.get() {
            on_assign.call((scene_id, position));
        }

        set_dragging_scene_id.set(None);
    };

    // Handle starting to drag a scene
    let handle_drag_start = move |scene_id: String, event: DragEvent| {
        if let Some(data_transfer) = event.data_transfer() {
            let _ = data_transfer.set_data("text/plain", &scene_id);
            set_dragging_scene_id.set(Some(scene_id));
        }
    };

    view! {
        <div class="scene-grid">
            {move || grid.get().into_iter().map(|(position, scene, is_active)| {
                let position_clone = position;
                
                let class = match (scene, is_active) {
                    (Some(_), true) => "scene-pad active",
                    (Some(s), _) if s.transition => "scene-pad transition",
                    (Some(_), _) => "scene-pad",
                    (None, _) => "scene-pad empty",
                };
                
                let style = scene.and_then(|s| s.color.map(|(r, g, b)| {
                    format!("background-color: rgb({}, {}, {});", r, g, b)
                })).unwrap_or_default();
                
                let on_click = move |_| {
                    if let Some(s) = scene {
                        on_activate.call(s.id.clone());
                    }
                };
                
                view! {
                    <div 
                        class=class
                        style=style
                        on:click=on_click
                        on:dragover=move |e: DragEvent| e.prevent_default()
                        on:drop=move |e| handle_drop(position_clone, e)
                        draggable="true"
                        on:dragstart=move |e| {
                            if let Some(s) = scene {
                                handle_drag_start(s.id.clone(), e);
                            }
                        }
                    >
                        <div class="scene-number">{position_clone + 1}</div>
                        {move || scene.map(|s| view! {
                            <div class="scene-name">{s.name.clone()}</div>
                        })}
                    </div>
                }}).collect::<Vec<_>>()
            }
        </div>
    }
}