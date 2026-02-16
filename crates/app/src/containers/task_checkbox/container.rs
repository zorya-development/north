use leptos::prelude::*;
use north_stores::use_app_store;

use super::controller::TaskCheckboxController;
use super::view::TaskCheckboxView;

#[component]
pub fn TaskCheckbox(task_id: i64) -> impl IntoView {
    let app_store = use_app_store();
    let ctrl = TaskCheckboxController::new(app_store, task_id);

    view! {
        <TaskCheckboxView
            is_completed=ctrl.is_completed
            progress=ctrl.progress
            on_toggle=Callback::new(move |()| ctrl.toggle_complete())
        />
    }
}
