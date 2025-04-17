use crate::models::{CCDefinition, CCValue, TransitionCurve};
use leptos::prelude::*;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, HtmlSelectElement};

#[component]
pub fn CCEditor(
    value: CCValue,
    definition: Option<CCDefinition>,
    is_editing: ReadSignal<bool>,
    on_change: Callback<CCValue>,
) -> impl IntoView {
    /* ---------- mutable signals for live editing ---------- */
    let (cur, set_cur) = create_signal(value.value);
    let (tx, set_tx) = create_signal(value.transition);
    let (beats, set_beats) = create_signal(value.transition_beats.unwrap_or(1.0));
    let (curve, set_curve) = create_signal(value.curve);

    /* ---------- derived info ---------- */
    let def_name = definition.as_ref().map(|d| d.name.clone());
    let def_desc = definition.as_ref().and_then(|d| d.description.clone());

    let name_str = value.name.clone();
    let desc_str = value.description.clone();

    let name = move || {
        name_str
            .clone()
            .or_else(|| def_name.clone())
            .unwrap_or_else(|| format!("CC {}", value.cc_number))
    };
    let desc = move || {
        desc_str
            .clone()
            .or_else(|| def_desc.clone())
            .unwrap_or_default()
    };

    let min = definition.as_ref().map(|d| d.min_value).unwrap_or(0);
    let max = definition.as_ref().map(|d| d.max_value).unwrap_or(127);
    let pct = create_memo(move |_| ((cur.get() - min) as f32 / (max - min) as f32) * 100.0);

    /* ---------- apply helper ---------- */
    let apply: Rc<dyn Fn()> = {
        let on_change = on_change.clone();
        let cur = cur.clone();
        let tx = tx.clone();
        let beats = beats.clone();
        let curve = curve.clone();
        let base = value.clone();
        Rc::new(move || {
            let mut nv = base.clone();
            nv.value = cur.get();
            nv.transition = tx.get();
            nv.transition_beats = if tx.get() { Some(beats.get()) } else { None };
            nv.curve = curve.get();
            on_change.run(nv);
        })
    };

    /* ---------- handlers ---------- */
    let handle_val = {
        let apply = apply.clone();
        move |e: leptos::ev::Event| {
            if let Ok(v) = event_target::<HtmlInputElement>(&e).value().parse::<u8>() {
                set_cur.set(v);
                apply();
            }
        }
    };

    let handle_tx = {
        let apply = apply.clone();
        move |e: leptos::ev::Event| {
            set_tx.set(event_target::<HtmlInputElement>(&e).checked());
            apply();
        }
    };

    let handle_beats = {
        let apply = apply.clone();
        move |e: leptos::ev::Event| {
            if let Ok(b) = event_target::<HtmlSelectElement>(&e).value().parse::<f32>() {
                set_beats.set(b);
                apply();
            }
        }
    };

    let handle_curve = {
        let apply = apply.clone();
        move |e: leptos::ev::Event| {
            let cv = match event_target::<HtmlSelectElement>(&e).value().as_str() {
                "linear" => TransitionCurve::Linear,
                "exponential" => TransitionCurve::Exponential,
                "logarithmic" => TransitionCurve::Logarithmic,
                "scurve" => TransitionCurve::SCurve,
                _ => TransitionCurve::Linear,
            };
            set_curve.set(cv);
            apply();
        }
    };

    /* ---------- view ---------- */
    view! {
        <div class="cc-editor">
            <div class="cc-header">
                <div class="cc-name">{name()}</div>
                <div class="cc-channel-info">
                    {format!("Ch: {}, CC: {}", value.channel + 1, value.cc_number)}
                </div>
            </div>

            <div class="cc-value-display">
                <div class="value-bar-container">
                    <div class="value-bar"
                         style=move || format!("width:{:.1}%;", pct.get()) />
                </div>
                <div class="value-text">{move || cur.get().to_string()}</div>
            </div>

            /* ---------- editing controls ---------- */
            <div class="cc-controls"
                 style=move || if is_editing.get() {""} else {"display:none;"}>
                <input type="range"
                       min=min max=max
                       prop:value=cur
                       on:input=handle_val />

                <label>
                    <input type="checkbox"
                           checked=tx
                           on:change=handle_tx />
                    "Transition"
                </label>

                <div class="transition-options"
                     style=move || if tx.get() {""} else {"display:none;"}>
                    <div class="transition-duration">
                        <label for="beats">"Beats:"</label>
                        <select id="beats"
                                prop:value=move || beats.get().to_string()
                                on:change=handle_beats>
                            {[
                                0.25, 0.5, 1.0, 2.0, 4.0, 8.0
                            ].iter().map(|b| view!{
                                <option value={b.to_string()}
                                        selected=move || (beats.get() - b).abs() < f32::EPSILON>
                                    {format!("{b}")}
                                </option>
                            }).collect::<Vec<_>>()}
                        </select>
                    </div>

                    <div class="transition-curve">
                        <label for="curve">"Curve:"</label>
                        <select id="curve"
                                prop:value=move || match curve.get() {
                                    TransitionCurve::Linear       => "linear",
                                    TransitionCurve::Exponential  => "exponential",
                                    TransitionCurve::Logarithmic  => "logarithmic",
                                    TransitionCurve::SCurve       => "scurve",
                                }
                                on:change=handle_curve>
                            {["linear","exponential","logarithmic","scurve"].iter().map(|v| view!{
                                <option value=*v
                                        selected=move || match (curve.get(), *v) {
                                            (TransitionCurve::Linear,      "linear")       => true,
                                            (TransitionCurve::Exponential, "exponential")  => true,
                                            (TransitionCurve::Logarithmic, "logarithmic")  => true,
                                            (TransitionCurve::SCurve,      "scurve")       => true,
                                            _ => false,
                                        }>
                                    {v.chars().next().unwrap().to_uppercase().collect::<String>() + &v[1..]}
                                </option>
                            }).collect::<Vec<_>>()}
                        </select>
                    </div>
                </div>
            </div>

            /* ---------- read‑only view ---------- */
            <div class="cc-info"
                 style=move || if !is_editing.get() {""} else {"display:none;"}>
                <div class="transition-info"
                     style=move || if value.transition {""} else {"display:none;"}>
                    {let cname = match value.curve {
                        TransitionCurve::Linear       => "Linear",
                        TransitionCurve::Exponential  => "Exponential",
                        TransitionCurve::Logarithmic  => "Logarithmic",
                        TransitionCurve::SCurve       => "S‑Curve",
                    };
                    let b = value.transition_beats.unwrap_or(1.0);
                    format!("Transition: {b} beats ({cname})")}
                </div>
                <div class="cc-description"
                     style=move || if desc().is_empty() {"display:none;"} else {""}>
                    {desc()}
                </div>
            </div>
        </div>
    }
}

/* tiny helper */
fn event_target<T: JsCast>(ev: &leptos::ev::Event) -> T {
    ev.target().unwrap().unchecked_into()
}
