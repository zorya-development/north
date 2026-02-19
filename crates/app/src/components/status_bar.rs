use leptos::prelude::*;
use north_stores::status_bar_store::{StatusBarStyle, StatusBarVariant};
use north_stores::use_app_store;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

const FRAMES: &[&str] = &["✦", "✧", "✶", "✷", "✸", "✹", "✸", "✷", "✶", "✧"];
const TOAST_DURATION_MS: i32 = 10_000;

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

    // Auto-dismiss toast messages after TOAST_DURATION_MS.
    Effect::new(move || {
        let msg = message.get();
        if let Some(ref m) = msg {
            if m.style == StatusBarStyle::Toast {
                let store = app_store.status_bar;
                let cb = Closure::once(Box::new(move || {
                    store.hide_message();
                }) as Box<dyn FnOnce()>);
                let _ = web_sys::window()
                    .unwrap()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        cb.as_ref().unchecked_ref(),
                        TOAST_DURATION_MS,
                    );
                cb.forget();
            }
        }
    });

    let variant_class = Memo::new(move |_| {
        message.get().map(|m| match m.variant {
            StatusBarVariant::Danger => "bg-danger text-on-accent",
            StatusBarVariant::Info => "bg-accent text-on-accent",
            StatusBarVariant::Success => "bg-success text-on-accent",
        })
    });

    let is_spinner = Memo::new(move |_| {
        message
            .get()
            .map(|m| m.style == StatusBarStyle::Spinner)
            .unwrap_or(false)
    });

    view! {
        <Show when=move || message.get().is_some()>
            <div class=move || {
                format!(
                    "status-bar fixed bottom-0 left-0 right-0 z-50 \
                     flex items-center px-4 py-1.5 \
                     font-mono text-sm shadow-lg {}",
                    variant_class.get().unwrap_or("")
                )
            }>
                <Show when=move || is_spinner.get()>
                    <span class="status-bar-spinner mr-3">
                        {move || FRAMES[frame_idx.get()]}
                    </span>
                </Show>
                <span class="flex-1">
                    {move || message.get().map(|m| m.text)}
                </span>
            </div>
        </Show>
    }
}
