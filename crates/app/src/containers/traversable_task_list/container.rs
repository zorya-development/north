use leptos::prelude::*;
use north_stores::use_app_store;

use super::controller::TraversableTaskListController;
use super::toolbar_config::ToolbarConfig;
use super::view::TraversableTaskListView;
use crate::components::drag_drop::DragDropContext;
use crate::containers::task_list_item::ItemConfig;

#[derive(Clone, Copy)]
pub struct ExtraVisibleIds(pub RwSignal<Vec<i64>>);

#[component]
pub fn TraversableTaskList(
    root_task_ids: Memo<Vec<i64>>,
    #[prop(default = ItemConfig::default())] item_config: ItemConfig,
    #[prop(default = "No tasks.")] empty_message: &'static str,
    #[prop(optional)] on_task_click: Option<Callback<i64>>,
    #[prop(default = Callback::new(|_| {}))] on_reorder: Callback<(
        i64,
        String,
        Option<Option<i64>>,
    )>,
    is_loaded: Signal<bool>,
    #[prop(default = true)] allow_create: bool,
    #[prop(default = true)] allow_reorder: bool,
    #[prop(optional)] default_project_id: Option<Signal<Option<i64>>>,
    #[prop(optional)] default_parent_id: Option<Signal<Option<i64>>>,
    #[prop(default = false)] flat: bool,
    #[prop(default = false)] scoped: bool,
    #[prop(optional)] cursor_task_id: Option<RwSignal<Option<i64>>>,
    #[prop(optional)] node_filter: Option<Signal<Callback<north_stores::TaskModel, bool>>>,
    #[prop(default = ToolbarConfig::none())] toolbar: ToolbarConfig,
    #[prop(optional)] show_keybindings_help: Option<RwSignal<bool>>,
) -> impl IntoView {
    let app_store = use_app_store();
    if item_config.draggable {
        provide_context(DragDropContext::new());
    }
    provide_context(ExtraVisibleIds(RwSignal::new(vec![])));

    let show_keybindings_help = show_keybindings_help.unwrap_or_else(|| RwSignal::new(false));

    let ctrl = TraversableTaskListController::new(
        app_store,
        app_store.modal,
        root_task_ids,
        show_keybindings_help,
        on_task_click,
        on_reorder,
        allow_create,
        allow_reorder,
        item_config,
        default_project_id,
        default_parent_id,
        flat,
        scoped,
        cursor_task_id,
        node_filter,
    );

    view! {
        <TraversableTaskListView
            ctrl=ctrl
            item_config=item_config
            empty_message=empty_message
            is_loaded=is_loaded
            scoped=scoped
            toolbar=toolbar
        />
    }
}
