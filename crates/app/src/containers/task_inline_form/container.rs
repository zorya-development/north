use leptos::prelude::*;
use north_stores::use_app_store;

use super::controller::TaskInlineFormController;
use super::view::TaskInlineFormView;

#[component]
pub fn TaskInlineForm(
    #[prop(optional)] task_id: Option<i64>,
    #[prop(optional)] project_id: Option<i64>,
    #[prop(optional)] parent_id: Option<i64>,
    on_done: Callback<()>,
) -> impl IntoView {
    let app_store = use_app_store();
    let ctrl = TaskInlineFormController::new(app_store, task_id, project_id, parent_id, on_done);

    view! { <TaskInlineFormView ctrl=ctrl/> }
}
