use leptos::prelude::*;
use leptos::*;

#[component]
pub fn ErrorDialog(message: String, on_close: Callback<()>) -> impl IntoView {
    view! {
        <div class="dialog-overlay">
            <div class="dialog error-dialog">
                <div class="dialog-header">
                    <h2>"Error"</h2>
                    <button
                        class="close-button"
                        on:click=move |_| on_close.clone().run(())
                    >
                        "×"
                    </button>
                </div>
                <div class="dialog-content">
                    <p class="error-message">{message}</p>
                </div>
                <div class="dialog-footer">
                    <button
                        class="button primary"
                        on:click=move |_| on_close.clone().run(())
                    >
                        "Close"
                    </button>
                </div>
            </div>
        </div>
    }
}
