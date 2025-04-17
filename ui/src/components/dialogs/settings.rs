use crate::models::Project;
use leptos::prelude::*;

#[component]
pub fn SettingsPanel(
    project: Option<Project>,
    on_close: Callback<()>,
    _on_save: Callback<Project>, // reserved
) -> impl IntoView {
    let project_signal = create_rw_signal(project);

    /* inner view – called only when project is Some */
    let quick_settings = move || {
        let p = project_signal.get().unwrap();

        view! {
            <div class="quick-settings">
                <div class="settings-item">
                    <span class="settings-label">"Tempo:"</span>
                    <span class="settings-value">
                        {format!("{} BPM", p.settings.default_tempo)}
                    </span>
                </div>
                <div class="settings-item">
                    <span class="settings-label">"Link:"</span>
                    <span class="settings-value">
                        {if p.settings.use_link { "Enabled" } else { "Disabled" }}
                    </span>
                </div>
                <div class="settings-item">
                    <span class="settings-label">"Quantization:"</span>
                    <span class="settings-value">
                        {match p.settings.default_quantization {
                            Some(1) => "1 Beat".into(),
                            Some(2) => "2 Beats".into(),
                            Some(4) => "1 Bar".into(),
                            Some(8) => "2 Bars".into(),
                            Some(n) => format!("{n} Beats"),
                            None    => "Off".into(),
                        }}
                    </span>
                </div>
            </div>
        }
    };

    /* panel */
    view! {
        <div class="settings-panel">
            <div class="panel-header">
                <h2>"Quick Settings"</h2>
                <button class="button-small"
                        on:click=move |_| on_close.clone().run(()) >
                    "Full Settings"
                </button>
            </div>

            <div class="panel-content">
                <Show when=move || project_signal.get().is_some()
                      fallback=|| view!{ <div class="no-project-message">
                          "Load a project to see settings"
                      </div> }>
                    {quick_settings}
                </Show>
            </div>
        </div>
    }
}
