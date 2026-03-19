use leptos::prelude::*;
use north_stores::use_app_store;

use super::controller::TraversableTaskListController;
use super::toolbar_config::ToolbarConfig;
use super::view::TraversableTaskListView;
use crate::components::drag_drop::DragDropContext;
use crate::containers::task_list_item::ItemConfig;
use crate::controllers::TaskTreeView;

#[derive(Clone, Copy)]
pub struct ExtraVisibleIds(pub RwSignal<Vec<i64>>);

#[component]
pub fn TraversableTaskList(
    /// Primary interface: provides tree, filter, extra_visible, is_loaded.
    /// When set, `root_task_ids`, `is_loaded`, and `flat` are ignored.
    #[prop(optional)]
    view: Option<TaskTreeView>,
    /// Flat-mode root IDs (used by filter page). Ignored when `view` is set.
    #[prop(optional)]
    root_task_ids: Option<Memo<Vec<i64>>>,
    #[prop(default = ItemConfig::default())] item_config: ItemConfig,
    #[prop(default = "No tasks.")] empty_message: &'static str,
    #[prop(optional)] on_task_click: Option<Callback<i64>>,
    #[prop(default = Callback::new(|_| {}))] on_reorder: Callback<(
        i64,
        String,
        Option<Option<i64>>,
    )>,
    /// Flat-mode is_loaded (used by filter page). Ignored when `view` is set.
    #[prop(optional)]
    is_loaded: Option<Signal<bool>>,
    #[prop(default = true)] allow_create: bool,
    #[prop(default = true)] allow_reorder: bool,
    #[prop(optional)] default_project_id: Option<Signal<Option<i64>>>,
    #[prop(optional)] default_parent_id: Option<Signal<Option<i64>>>,
    /// Flat display mode (no tree expansion). Used by filter page.
    #[prop(default = false)]
    flat: bool,
    #[prop(default = false)] scoped: bool,
    #[prop(optional)] cursor_task_id: Option<RwSignal<Option<i64>>>,
    /// Node filter for flat-mode consumers (filter page, task detail modal subtasks).
    /// Ignored when `view` is set.
    #[prop(optional)]
    node_filter: Option<Signal<Callback<north_stores::TaskModel, bool>>>,
    #[prop(default = ToolbarConfig::none())] toolbar: ToolbarConfig,
    #[prop(optional)] show_keybindings_help: Option<RwSignal<bool>>,
) -> impl IntoView {
    let app_store = use_app_store();
    if item_config.draggable {
        provide_context(DragDropContext::new());
    }

    let show_keybindings_help = show_keybindings_help.unwrap_or_else(|| RwSignal::new(false));

    let effective_is_loaded = if let Some(ref v) = view {
        v.is_loaded
    } else {
        is_loaded.unwrap_or_else(|| Signal::derive(|| true))
    };

    let extra_visible_ids = if let Some(ref v) = view {
        v.extra_visible
    } else {
        RwSignal::new(vec![])
    };
    provide_context(ExtraVisibleIds(extra_visible_ids));

    let ctrl = TraversableTaskListController::new(
        app_store,
        app_store.modal,
        view,
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
            is_loaded=effective_is_loaded
            scoped=scoped
            toolbar=toolbar
        />
    }
}
