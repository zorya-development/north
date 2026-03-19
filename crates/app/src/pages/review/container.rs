use leptos::prelude::*;
use north_stores::use_app_store;

use super::controller::ReviewController;
use super::view::ReviewView;

#[component]
pub fn ReviewPage() -> impl IntoView {
    let app_store = use_app_store();
    let ctrl = ReviewController::new(app_store);

    view! {
        <ReviewView
            pending_view=ctrl.pending_view
            reviewed_view=ctrl.reviewed_view
            show_reviewed=ctrl.show_reviewed.0
            set_show_reviewed=ctrl.show_reviewed.1
            on_task_click=Callback::new(move |id| ctrl.open_detail(id))
            toolbar=ctrl.toolbar_config()
            show_keybindings_help=RwSignal::new(false)
        />
    }
}
