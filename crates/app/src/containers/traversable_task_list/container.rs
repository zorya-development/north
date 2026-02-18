use leptos::prelude::*;
use north_stores::use_app_store;

use super::controller::TraversableTaskListController;
use super::view::TraversableTaskListView;
use crate::containers::task_list::ExtraVisibleIds;

#[component]
pub fn TraversableTaskList(
    root_task_ids: Memo<Vec<i64>>,
    show_completed: RwSignal<bool>,
    #[prop(default = true)] show_project: bool,
    #[prop(default = false)] draggable: bool,
    #[prop(default = "No tasks.")] empty_message: &'static str,
    #[prop(optional)] on_task_click: Option<Callback<i64>>,
    #[prop(default = Callback::new(|_| {}))] on_reorder: Callback<(
        i64,
        String,
        Option<Option<i64>>,
    )>,
    is_loaded: Signal<bool>,
    #[prop(optional)] show_keybindings_help: Option<RwSignal<bool>>,
    #[prop(default = true)] allow_create: bool,
    #[prop(default = true)] allow_reorder: bool,
    #[prop(default = false)] show_review: bool,
    #[prop(optional)] default_project_id: Option<Signal<Option<i64>>>,
    #[prop(default = false)] flat: bool,
) -> impl IntoView {
    let app_store = use_app_store();
    provide_context(ExtraVisibleIds(RwSignal::new(vec![])));

    let show_keybindings_help = show_keybindings_help.unwrap_or_else(|| RwSignal::new(false));

    let ctrl = TraversableTaskListController::new(
        app_store,
        app_store.modal,
        root_task_ids,
        show_completed,
        show_keybindings_help,
        on_task_click,
        on_reorder,
        allow_create,
        allow_reorder,
        show_review,
        default_project_id,
        flat,
    );

    view! {
        <TraversableTaskListView
            ctrl=ctrl
            show_project=show_project
            draggable=draggable
            empty_message=empty_message
            is_loaded=is_loaded
        />
    }
}
