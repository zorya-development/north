use leptos::ev;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use north_stores::{use_app_store, use_task_detail_modal_store};

use super::controller::TaskDetailModalController;
use super::view::TaskDetailModalView;
use crate::containers::task_list_item::ItemConfig;
use crate::containers::traversable_task_list::ExtraVisibleIds;

#[component]
pub fn TaskDetailModal() -> impl IntoView {
    let store = use_task_detail_modal_store();
    let app_store = use_app_store();
    let is_open = Memo::new(move |_| store.is_open());

    window_event_listener(ev::keydown, move |ev| {
        if ev.key() == "Escape" && is_open.get_untracked() {
            ev.prevent_default();
            // Blur active element before closing so that blur handlers fire
            // while signals are still alive (before <Show> disposes the scope).
            if let Some(el) = document().active_element() {
                if let Some(html_el) = el.dyn_ref::<web_sys::HtmlElement>() {
                    let _ = html_el.blur();
                }
            }
            store.close();
        }
    });

    view! {
        <Show when=move || is_open.get()>
            {
                provide_context(ExtraVisibleIds(RwSignal::new(vec![])));
                let ctrl = TaskDetailModalController::new(app_store);
                let subtask_item_config = ItemConfig {
                    show_project: false,
                    show_inline_tags: false,
                    ..Default::default()
                };

                view! {
                    <TaskDetailModalView
                        ctrl=ctrl
                        subtask_item_config=subtask_item_config
                    />
                }
            }
        </Show>
    }
}
