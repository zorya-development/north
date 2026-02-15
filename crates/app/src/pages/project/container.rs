use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use north_stores::use_app_store;

use super::controller::ProjectController;
use super::view::ProjectView;

#[component]
pub fn ProjectPage() -> impl IntoView {
    let app_store = use_app_store();
    let params = use_params_map();

    let project_id = Signal::derive(move || {
        params
            .read()
            .get("id")
            .and_then(|id| id.parse::<i64>().ok())
            .unwrap_or(0)
    });

    let ctrl = ProjectController::new(app_store, project_id);
    let (is_form_open, set_form_open) = ctrl.is_new_task_form_open;

    view! {
        <ProjectView
            project=ctrl.project
            project_id=ctrl.project_id
            active_task_ids=ctrl.active_task_ids
            completed_task_ids=ctrl.completed_task_ids
            completed_count=ctrl.completed_count
            is_loaded=ctrl.is_loaded
            is_form_open=is_form_open
            set_form_open=set_form_open
            on_task_click=Callback::new(move |id| ctrl.open_detail(id))
            on_reorder=Callback::new(move |(id, key, parent)| {
                ctrl.reorder_task(id, key, parent)
            })
            active_tasks_for_reorder=ctrl.active_tasks_for_reorder
        />
    }
}
