use leptos::prelude::*;
use north_stores::status_bar_store::StatusBarVariant;
use north_stores::use_app_store;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

const FRAMES: &[&str] = &["✦", "✧", "✶", "✷", "✸", "✹", "✸", "✷", "✶", "✧"];

#[component]
pub fn StatusBar() -> impl IntoView {
    let app_store = use_app_store();
    let message = app_store.status_bar.message;
    let (frame_idx, set_frame_idx) = signal(0_usize);

    // Client-only effect — set_interval panics during SSR.
    Effect::new(move || {
        let cb = Closure::wrap(Box::new(move || {
            set_frame_idx.update(|i| *i = (*i + 1) % FRAMES.len());
        }) as Box<dyn FnMut()>);
        let _ = web_sys::window()
            .unwrap()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                120,
            );
        cb.forget();
    });

    let is_danger = Memo::new(move |_| {
        message
            .get()
            .map(|m| m.variant == StatusBarVariant::Danger)
            .unwrap_or(false)
    });

    view! {
        <Show when=move || message.get().is_some()>
            <div class=move || {
                if is_danger.get() {
                    "status-bar fixed bottom-0 left-0 right-0 z-50 \
                     flex items-center px-4 py-1.5 \
                     font-mono text-sm shadow-lg \
                     bg-danger text-on-accent"
                } else {
                    "status-bar fixed bottom-0 left-0 right-0 z-50 \
                     flex items-center px-4 py-1.5 \
                     font-mono text-sm shadow-lg \
                     bg-accent text-on-accent"
                }
            }>
                <span class="status-bar-spinner mr-3">
                    {move || FRAMES[frame_idx.get()]}
                </span>
                <span class="flex-1">
                    {move || message.get().map(|m| m.text)}
                </span>
            </div>
        </Show>
    }
}
