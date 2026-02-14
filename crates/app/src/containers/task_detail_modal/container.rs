use leptos::prelude::*;
use north_stores::use_task_detail_modal_store;

use super::view::TaskDetailModalView;

#[component]
pub fn TaskDetailModal() -> impl IntoView {
    let store = use_task_detail_modal_store();
    let is_open = Memo::new(move |_| store.is_open());

    view! {
        <Show when=move || is_open.get()>
            <TaskDetailModalView store=store/>
        </Show>
    }
}
