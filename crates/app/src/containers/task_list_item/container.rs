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
    #[prop(default = false)] compact: bool,
    #[prop(default = 0)] depth: u8,
    #[prop(optional)] on_click: Option<Callback<i64>>,
) -> impl IntoView {
    let app_store = use_app_store();
    let ctrl = TaskListItemController::new(app_store, task_id);

    view! {
        <TaskListItemView
            task=ctrl.task
            show_review=show_review
            show_project=show_project
            draggable=draggable
            compact=compact
            depth=depth
            on_click=on_click
            on_delete=Callback::new(move |()| ctrl.delete())
            on_review=Callback::new(move |()| ctrl.review())
            on_set_start_at=Callback::new(move |sa| ctrl.set_start_at(sa))
            on_clear_start_at=Callback::new(move |()| ctrl.clear_start_at())
            on_set_project=Callback::new(move |pid| ctrl.set_project(pid))
            on_clear_project=Callback::new(move |()| ctrl.clear_project())
            on_set_tags=Callback::new(move |tags| ctrl.set_tags(tags))
        />
    }
}
