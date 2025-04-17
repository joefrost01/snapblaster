use leptos::*;
use leptos::prelude::{create_memo, create_signal, Callback, ClassAttribute, Get, GlobalAttributes, OnAttribute, ReadSignal, Set, StyleAttribute};
use wasm_bindgen::JsCast;
use crate::models::{CCValue, CCDefinition, TransitionCurve};

#[component]
pub fn CCEditor(
    value: CCValue,
    definition: Option<CCDefinition>,
    is_editing: ReadSignal<bool>,
    on_change: Callback<CCValue>,
) -> impl IntoView {
    // Local state for editing
    let (current_value, set_current_value) = create_signal(value.value);
    let (transition_enabled, set_transition_enabled) = create_signal(value.transition);
    let (transition_beats, set_transition_beats) = create_signal(value.transition_beats.unwrap_or(1.0));
    let (curve_type, set_curve_type) = create_signal(value.curve);

    // Get display name from either the CC value or the definition
    let display_name = move || {
        value.name.clone().unwrap_or_else(|| {
            definition.as_ref().map(|def| def.name.clone())
                .unwrap_or_else(|| format!("CC {}", value.cc_number))
        })
    };

    // Get value description
    let description = move || {
        value.description.clone().unwrap_or_else(|| {
            definition.as_ref().and_then(|def| def.description.clone())
                .unwrap_or_default()
        })
    };

    // Get min and max values from definition or defaults
    let min_value = definition.as_ref().map(|def| def.min_value).unwrap_or(0);
    let max_value = definition.as_ref().map(|def| def.max_value).unwrap_or(127);

    // Value as percentage (for progress bar visualization)
    let value_percent = create_memo(move |_| {
        let range = max_value - min_value;
        if range == 0 {
            return 0.0;
        }

        let relative_value = current_value.get() - min_value;
        (relative_value as f32 / range as f32) * 100.0
    });

    // Apply changes to the CC value
    let apply_changes = move || {
        let mut new_value = value.clone();
        new_value.value = current_value.get();
        new_value.transition = transition_enabled.get();

        if transition_enabled.get() {
            new_value.transition_beats = Some(transition_beats.get());
            new_value.curve = curve_type.get();
        } else {
            new_value.transition_beats = None;
        }

        on_change.call(new_value);
    };

    view! {
        <div class="cc-editor">
            <div class="cc-header">
                <div class="cc-name">{display_name}</div>
                <div class="cc-channel-info">
                    {format!("Ch: {}, CC: {}", value.channel + 1, value.cc_number)}
                </div>
            </div>
            
            <div class="cc-value-display">
                <div class="value-bar-container">
                    <div 
                        class="value-bar" 
                        style=move || format!("width: {}%;", value_percent.get())
                    ></div>
                </div>
                <div class="value-text">{move || current_value.get()}</div>
            </div>
            
            {move || if is_editing.get() {
                view! {
                    <div class="cc-controls">
                        <input 
                            type="range"
                            min=min_value
                            max=max_value
                            value=current_value
                            on:input=move |e| {
                                let target = event_target::<web_sys::HtmlInputElement>(&e);
                                if let Ok(value) = target.value().parse::<u8>() {
                                    set_current_value.set(value);
                                    apply_changes();
                                }
                            }
                        />
                        
                        <div class="transition-controls">
                            <label>
                                <input 
                                    type="checkbox"
                                    checked=transition_enabled
                                    on:change=move |e| {
                                        let target = event_target::<web_sys::HtmlInputElement>(&e);
                                        set_transition_enabled.set(target.checked());
                                        apply_changes();
                                    }
                                />
                                "Transition"
                            </label>
                            
                            {move || if transition_enabled.get() {
                                view! {
                                    <div class="transition-options">
                                        <div class="transition-duration">
                                            <label for="beats">"Beats:"</label>
                                            <select 
                                                id="beats"
                                                on:change=move |e| {
                                                    let target = event_target::<web_sys::HtmlSelectElement>(&e);
                                                    if let Ok(beats) = target.value().parse::<f32>() {
                                                        set_transition_beats.set(beats);
                                                        apply_changes();
                                                    }
                                                }
                                            >
                                                <option value="0.25" selected=move || (transition_beats.get() - 0.25).abs() < 0.01>
                                                    "1/4"
                                                </option>
                                                <option value="0.5" selected=move || (transition_beats.get() - 0.5).abs() < 0.01>
                                                    "1/2"
                                                </option>
                                                <option value="1.0" selected=move || (transition_beats.get() - 1.0).abs() < 0.01>
                                                    "1"
                                                </option>
                                                <option value="2.0" selected=move || (transition_beats.get() - 2.0).abs() < 0.01>
                                                    "2"
                                                </option>
                                                <option value="4.0" selected=move || (transition_beats.get() - 4.0).abs() < 0.01>
                                                    "4"
                                                </option>
                                                <option value="8.0" selected=move || (transition_beats.get() - 8.0).abs() < 0.01>
                                                    "8"
                                                </option>
                                            </select>
                                        </div>
                                        
                                        <div class="transition-curve">
                                            <label for="curve">"Curve:"</label>
                                            <select 
                                                id="curve"
                                                on:change=move |e| {
                                                    let target = event_target::<web_sys::HtmlSelectElement>(&e);
                                                    let curve = match target.value().as_str() {
                                                        "linear" => TransitionCurve::Linear,
                                                        "exponential" => TransitionCurve::Exponential,
                                                        "logarithmic" => TransitionCurve::Logarithmic,
                                                        "scurve" => TransitionCurve::SCurve,
                                                        _ => TransitionCurve::Linear,
                                                    };
                                                    set_curve_type.set(curve);
                                                    apply_changes();
                                                }
                                            >
                                                <option value="linear" selected=move || matches!(curve_type.get(), TransitionCurve::Linear)>
                                                    "Linear"
                                                </option>
                                                <option value="exponential" selected=move || matches!(curve_type.get(), TransitionCurve::Exponential)>
                                                    "Exponential"
                                                </option>
                                                <option value="logarithmic" selected=move || matches!(curve_type.get(), TransitionCurve::Logarithmic)>
                                                    "Logarithmic"
                                                </option>
                                                <option value="scurve" selected=move || matches!(curve_type.get(), TransitionCurve::SCurve)>
                                                    "S-Curve"
                                                </option>
                                            </select>
                                        </div>
                                    </div>
                                }
                            } else {
                                view! { <div></div> }
                            }}
                        </div>
                    </div>
                }
            } else {
                view! {
                    <div class="cc-info">
                        {move || if value.transition {
                            let curve_name = match value.curve {
                                TransitionCurve::Linear => "Linear",
                                TransitionCurve::Exponential => "Exponential",
                                TransitionCurve::Logarithmic => "Logarithmic",
                                TransitionCurve::SCurve => "S-Curve",
                            };
                            
                            let beats = value.transition_beats.unwrap_or(1.0);
                            
                            view! {
                                <div class="transition-info">
                                    {"Transition: "}
                                    {format!("{} beats ({})", beats, curve_name)}
                                </div>
                            }
                        } else {
                            view! {}
                        }}
                        
                        {move || if !description().is_empty() {
                            view! { <div class="cc-description">{description()}</div> }
                        } else {
                            view! {}
                        }}
                    </div>
                }
            }}
        </div>
    }
}

// Helper function to get the event target
fn event_target<T: wasm_bindgen::JsCast>(event: &leptos::ev::Event) -> T {
    event.target().unwrap().unchecked_into::<T>()
}