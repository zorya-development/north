pub mod app;
pub mod components;
pub mod pages;
pub mod server_fns;
pub mod util;

pub use app::*;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    leptos::mount::hydrate_body(App);
}
