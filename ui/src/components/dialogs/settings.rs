use leptos::*;
use leptos::prelude::{Callback, ClassAttribute, OnAttribute};
use crate::models::Project;

#[component]
pub fn SettingsPanel(
    #[prop(optional)]
    project: Option<Project>,
    on_open_settings: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="settings-panel">
            <div class="panel-header">
                <h2>"Quick Settings"</h2>
                <button
                    class="button-small"
                    on:click=move |_| on_open_settings.call(())
                >
                    "Full Settings"
                </button>
            </div>
            
            <div class="panel-content">
                {move || match project.clone() {
                    Some(p) => view! {
                        <div class="quick-settings">
                            <div class="settings-item">
                                <span class="settings-label">"Tempo:"</span>
                                <span class="settings-value">{p.settings.default_tempo.to_string()}{" BPM"}</span>
                            </div>
                            
                            <div class="settings-item">
                                <span class="settings-label">"Link:"</span>
                                <span class="settings-value">{if p.settings.use_link { "Enabled" } else { "Disabled" }}</span>
                            </div>
                            
                            <div class="settings-item">
                                <span class="settings-label">"Quantization:"</span>
                                <span class="settings-value">
                                    {match p.settings.default_quantization {
                                        Some(1) => "1 Beat",
                                        Some(2) => "2 Beats",
                                        Some(4) => "1 Bar",
                                        Some(8) => "2 Bars",
                                        Some(n) => format!("{} Beats", n),
                                        None => "Off"
                                    }}
                                </span>
                            </div>
                        </div>
                    },
                    None => view! {
                        <div class="no-project-message">
                            "Load a project to see settings"
                        </div>
                    }
                }}
            </div>
        </div>
    }
}