use crate::components::CCEditor;
use crate::models::{CCDefinition, Scene, TriggerMode};
use leptos::prelude::*;
use leptos::*;
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

#[component]
pub fn SceneEditor(
    scene: Scene,
    cc_definitions: HashMap<String, CCDefinition>,
    on_update: Callback<Scene>,
) -> impl IntoView {
    // ------------- signals -------------
    let (name, set_name) = create_signal(scene.name.clone());
    let (desc, set_desc) = create_signal(scene.description.clone().unwrap_or_default());
    let (mode, set_mode) = create_signal(scene.trigger_mode.clone());
    let (cc_vals, set_vals) = create_signal(scene.cc_values.clone());
    let (is_edit, set_edit) = create_signal(false);
    let (dirty, set_dirty) = create_signal(false);

    let scene_orig = scene.clone();

    // ------------- save / cancel -------------
    let save = move |_| {
        let mut s2 = scene.clone();
        s2.name = name.get();
        s2.description = Some(desc.get()).filter(|s| !s.is_empty());
        s2.trigger_mode = mode.get();
        s2.cc_values = cc_vals.get();
        on_update.run(s2);
        set_edit.set(false);
        set_dirty.set(false);
    };

    let cancel = move |_| {
        set_edit.set(false);
        set_name.set(scene_orig.name.clone());
        set_desc.set(scene_orig.description.clone().unwrap_or_default());
        set_mode.set(scene_orig.trigger_mode.clone());
        set_vals.set(scene_orig.cc_values.clone());
        set_dirty.set(false);
    };

    // ------------- view -------------
    view! {
        <div class="scene-editor-container">
            /* header */
            <div class="scene-editor-header">
                <div class="edit-header"
                     style=move || if is_edit.get() { "" } else { "display:none;" }>
                    <input type="text"
                           value=name
                           placeholder="Scene Name"
                           on:input=move |e| {
                               set_name.set(event_target::<HtmlInputElement>(&e).value());
                               set_dirty.set(true);
                           } />
                    <div class="edit-actions">
                        <button on:click=save
                                disabled=move || !dirty.get()
                                class="save-button">"Save"</button>
                        <button on:click=cancel class="cancel-button">"Cancel"</button>
                    </div>
                </div>

                <div class="view-header"
                     style=move || if !is_edit.get() { "" } else { "display:none;" }>
                    <h2>{ name }</h2>
                    <button on:click=move |_| set_edit.set(true)
                            class="edit-button">"Edit"</button>
                </div>
            </div>

            /* meta + CC list */
            <div class="scene-editor-details">
                <div class="scene-meta">
                    <div class="detail-row">
                        <span class="label">"Trigger Mode:"</span>
                        <span class="value">
                            {move || match mode.get() {
                                TriggerMode::Immediate => "Immediate",
                                TriggerMode::NextBeat  => "Next Beat",
                                TriggerMode::Beats(n)  => return format!("{n} Beats"),
                                TriggerMode::NextBar   => "Next Bar",
                            }.into()}
                        </span>
                    </div>

                    <div class="detail-row"
                         style=move || if desc.get().is_empty() { "display:none;" } else { "" }>
                        <span class="label">"Description:"</span>
                        <span class="value">{ move || desc.get() }</span>
                    </div>
                </div>

                <div class="cc-values-container">
                    <h3>"CC Values"</h3>

                    <div class="cc-list"
                         style=move || if cc_vals.get().is_empty() { "display:none;" } else { "" }>
                        {move || {
                            cc_vals.get()
                                   .iter()
                                   .filter_map(|(k, v)| {
                                       let mut parts = k.split(':');
                                       match (parts.next(), parts.next()) {
                                           (Some(ch), Some(ccn)) if ch.parse::<u8>().is_ok() && ccn.parse::<u8>().is_ok() => {
                                               Some((ch.parse::<u8>().unwrap(), ccn.parse::<u8>().unwrap(), v.clone()))
                                           },
                                           _ => None,
                                       }
                                   })
                                   .fold(HashMap::<u8, Vec<_>>::new(), |mut acc, (ch, ccn, v)| {
                                       acc.entry(ch).or_default().push((ccn, v));
                                       acc
                                   })
                                   .into_iter()
                                   .map(|(ch, mut vs)| { vs.sort_by_key(|(n, _)| *n); (ch, vs) })
                                   .map(|(ch, vs)| {
                                       let group = format!("Channel {ch}");
                                       view! {
                                           <div class="cc-group">
                                               <h4>{ group }</h4>
                                               <div class="cc-list">
                                                   {vs.into_iter().map(|(ccn, vv)| {
                                                       let key   = format!("{}:{}", vv.channel, ccn);
                                                       let def   = cc_definitions.get(&key).cloned();
                                                       let vv_cp = vv.clone();
                                                       view! {
                                                           <CCEditor value=vv
                                                                     definition=def
                                                                     is_editing=is_edit
                                                                     on_change=Callback::new(move |nv| {
                                                                         let mut m = cc_vals.get();
                                                                         m.insert(format!("{}:{}", vv_cp.channel, ccn), nv);
                                                                         set_vals.set(m);
                                                                         set_dirty.set(true);
                                                                     }) />
                                                       }
                                                   }).collect::<Vec<_>>()}
                                               </div>
                                           </div>
                                       }
                                   }).collect::<Vec<_>>()
                        }}
                    </div>

                    <p class="empty-message"
                       style=move || if cc_vals.get().is_empty() { "" } else { "display:none;" }>
                        "No CC values in this scene"
                    </p>
                </div>
            </div>
        </div>
    }
}

fn event_target<T: JsCast>(ev: &leptos::ev::Event) -> T {
    ev.target().unwrap().unchecked_into()
}
