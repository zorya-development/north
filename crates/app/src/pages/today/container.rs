use leptos::prelude::*;
use north_stores::use_app_store;

use super::controller::TodayController;
use super::view::TodayView;

#[component]
pub fn TodayPage() -> impl IntoView {
    let app_store = use_app_store();
    let ctrl = TodayController::new(app_store);

    view! {
        <TodayView
            root_task_ids=ctrl.root_task_ids
            show_completed=ctrl.show_completed
            completed_count=ctrl.completed_count
            is_loaded=ctrl.is_loaded
            hide_non_actionable=ctrl.hide_non_actionable
            on_task_click=Callback::new(move |id| ctrl.open_detail(id))
            on_toggle_visibility=Callback::new(move |()| {
                ctrl.toggle_actionable_visibility()
            })
        />
    }
}
