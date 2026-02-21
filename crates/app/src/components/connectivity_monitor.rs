use leptos::prelude::*;
use leptos::task::spawn_local;
use north_stores::status_bar_store::StatusBarVariant;
use north_stores::use_app_store;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

const PING_INTERVAL_MS: i32 = 30_000;

/// Headless component that periodically pings the server and shows
/// a persistent "Disconnected" banner via StatusBarStore when unreachable.
#[component]
pub fn ConnectivityMonitor() -> impl IntoView {
    let app_store = use_app_store();
    let was_disconnected = RwSignal::new(false);

    Effect::new(move || {
        let cb = Closure::wrap(Box::new(move || {
            spawn_local(async move {
                match north_server_fns::ping::ping().await {
                    Ok(()) => {
                        if was_disconnected.get_untracked() {
                            was_disconnected.set(false);
                            app_store.status_bar.hide_message();
                            app_store
                                .status_bar
                                .notify(StatusBarVariant::Success, "Reconnected to server");
                        }
                    }
                    Err(_) => {
                        if !was_disconnected.get_untracked() {
                            was_disconnected.set(true);
                            app_store
                                .status_bar
                                .show_message("Disconnected from server", StatusBarVariant::Danger);
                        }
                    }
                }
            });
        }) as Box<dyn FnMut()>);

        let window = web_sys::window().unwrap();
        let id = window
            .set_interval_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                PING_INTERVAL_MS,
            )
            .unwrap();
        cb.forget();

        on_cleanup(move || {
            if let Some(w) = web_sys::window() {
                w.clear_interval_with_handle(id);
            }
        });
    });
}
