use leptos::prelude::*;
use north_stores::use_app_store;

use super::controller::TaskListItemController;
use super::view::TaskListItemView;

#[component]
pub fn TaskListItem(
    task_id: i64,
    #[prop(default = false)] show_review: bool,
    #[prop(default = true)] show_project: bool,
    #[prop(default = false)] draggable: bool,
    #[prop(default = 0)] depth: u8,
    #[prop(optional)] on_click: Option<Callback<i64>>,
) -> impl IntoView {
    let app_store = use_app_store();
    let ctrl = TaskListItemController::new(app_store, task_id);

    view! {
        <TaskListItemView
            ctrl=ctrl
            show_review=show_review
            show_project=show_project
            draggable=draggable
            depth=depth
            on_click=on_click
        />
    }
}
