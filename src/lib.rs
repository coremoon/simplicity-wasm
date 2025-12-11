pub mod wasm_api;

use leptos::{mount_to_body, view};

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <h1>"Simplicity Compiler"</h1> })
}
