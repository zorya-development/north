use leptos::prelude::*;
use north_stores::use_app_store;

use super::controller::TaskListItemController;
use super::view::TaskListItemView;

#[derive(Clone, Copy)]
pub struct ItemConfig {
    pub show_review: bool,
    pub show_project: bool,
    pub draggable: bool,
}

impl Default for ItemConfig {
    fn default() -> Self {
        Self {
            show_review: false,
            show_project: true,
            draggable: false,
        }
    }
}

#[component]
pub fn TaskListItem(
    task_id: i64,
    #[prop(default = ItemConfig::default())] config: ItemConfig,
) -> impl IntoView {
    let app_store = use_app_store();
    let ctrl = TaskListItemController::new(app_store, task_id);

    view! {
        <TaskListItemView
            task=ctrl.task
            show_review=config.show_review
            show_project=config.show_project
            draggable=config.draggable
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
