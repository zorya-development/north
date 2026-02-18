use leptos::ev;
use leptos::prelude::*;
use north_stores::use_task_detail_modal_store;

use super::view::TaskDetailModalView;
use crate::containers::task_list::ExtraVisibleIds;

#[component]
pub fn TaskDetailModal() -> impl IntoView {
    let store = use_task_detail_modal_store();
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
                view! { <TaskDetailModalView store=store/> }
            }
        </Show>
    }
}
