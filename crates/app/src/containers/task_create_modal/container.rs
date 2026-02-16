use leptos::prelude::*;
use north_stores::TaskCreateModalStore;

use super::view::TaskCreateModalView;

#[component]
pub fn TaskCreateModal() -> impl IntoView {
    let store = expect_context::<TaskCreateModalStore>();

    view! {
        <Show when=move || store.is_open()>
            <TaskCreateModalView store=store/>
        </Show>
    }
}
