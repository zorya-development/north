#![recursion_limit = "256"]

pub mod app;
pub mod atoms;
pub mod components;
pub mod constants;
pub mod containers;
pub mod libs;
pub mod pages;

pub use app::*;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
