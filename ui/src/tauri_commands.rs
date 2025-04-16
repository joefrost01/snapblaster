use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use js_sys::{Object, Reflect};
use serde::{Serialize, Deserialize};
use leptos::*;

// Helper function to invoke Tauri commands
pub async fn invoke<T, R>(command: &str, args: Option<T>) -> Result<R, String>
where
    T: Serialize,
    R: for<'de> Deserialize<'de>,
{
    let window = web_sys::window().unwrap();

    // Access the `__TAURI__` object
    let tauri = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__"))
        .map_err(|_| "Tauri API not found".to_string())?;

    // Access the `invoke` function
    let invoke_fn = js_sys::Reflect::get(&tauri, &JsValue::from_str("invoke"))
        .map_err(|_| "Tauri invoke function not found".to_string())?;

    // Create the arguments
    let args_value = match args {
        Some(data) => serde_wasm_bindgen::to_value(&data)
            .map_err(|e| format!("Failed to serialize args: {:?}", e))?,
        None => JsValue::NULL,
    };

    // Call the invoke function
    let promise = js_sys::Reflect::apply(
        &invoke_fn.dyn_into::<js_sys::Function>().unwrap(),
        &tauri,
        &js_sys::Array::of3(
            &JsValue::from_str(command),
            &args_value,
            &JsValue::NULL, // No options
        ),
    ).map_err(|e| format!("Failed to call Tauri invoke: {:?}", e))?;

    // Wait for the promise to resolve
    let result = JsFuture::from(promise.dyn_into::<js_sys::Promise>().unwrap())
        .await
        .map_err(|e| format!("Tauri command failed: {:?}", e))?;

    // Deserialize the result
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to deserialize result: {:?}", e))
}

// Example command wrapper
pub async fn get_project() -> Result<Project, String> {
    invoke("get_project", None::<()>).await
}

// Add more command wrappers here...