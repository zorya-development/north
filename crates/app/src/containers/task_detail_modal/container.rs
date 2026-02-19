use leptos::ev;
use leptos::prelude::*;
use north_stores::{use_app_store, use_task_detail_modal_store};

use super::view::TaskDetailModalView;
use crate::containers::traversable_task_list::ExtraVisibleIds;

#[component]
pub fn TaskDetailModal() -> impl IntoView {
    let store = use_task_detail_modal_store();
    let app_store = use_app_store();
    let is_open = Memo::new(move |_| store.is_open());

    window_event_listener(ev::keydown, move |ev| {
        if ev.key() == "Escape" && is_open.get_untracked() {
            ev.prevent_default();
            store.close();
        }
    });

    view! {
        <Show when=move || is_open.get()>
            {
                provide_context(ExtraVisibleIds(RwSignal::new(vec![])));
                view! {
                    <TaskDetailModalView
                        store=store
                        on_recurrence_open=Callback::new(move |()| {
                            app_store.modal.open("recurrence");
                        })
                        on_recurrence_close=Callback::new(move |()| {
                            app_store.modal.close("recurrence");
                        })
                        show_recurrence_modal=Signal::derive(move || {
                            app_store.modal.is_open("recurrence")
                        })
                    />
                }
            }
        </Show>
    }
}
