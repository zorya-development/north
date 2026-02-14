use leptos::prelude::*;
use north_stores::use_app_store;

use super::controller::InboxController;
use super::view::InboxView;
use crate::components::drag_drop::DragDropContext;

#[component]
pub fn InboxPage() -> impl IntoView {
    let app_store = use_app_store();
    let ctrl = InboxController::new(app_store);

    provide_context(DragDropContext::new());

    view! { <InboxView ctrl=ctrl/> }
}
