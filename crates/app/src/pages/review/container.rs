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
            review_task_ids=ctrl.review_task_ids
            reviewed_task_ids=ctrl.reviewed_task_ids
            is_loaded=ctrl.is_loaded
            show_reviewed=ctrl.show_reviewed.0
            set_show_reviewed=ctrl.show_reviewed.1
            on_review_all=Callback::new(move |()| ctrl.review_all())
            on_task_click=Callback::new(move |id| ctrl.open_detail(id))
        />
    }
}
