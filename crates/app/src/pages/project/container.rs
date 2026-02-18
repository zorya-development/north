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

    let default_project_id = Signal::derive(move || Some(project_id.get()));

    view! {
        <ProjectView
            project=ctrl.project
            root_task_ids=ctrl.root_task_ids
            show_completed=ctrl.show_completed
            completed_count=ctrl.completed_count
            is_loaded=ctrl.is_loaded
            default_project_id=default_project_id
            on_add_task=Callback::new(move |()| ctrl.open_create())
            on_task_click=Callback::new(move |id| ctrl.open_detail(id))
            on_reorder=Callback::new(move |(id, key, parent)| {
                ctrl.reorder_task(id, key, parent)
            })
        />
    }
}
