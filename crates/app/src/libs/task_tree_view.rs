use leptos::prelude::*;
use north_stores::{AppStore, TaskModel, TaskTree};

/// Reactive filtered view over the shared TaskTree.
///
/// Pages create a TaskTreeView with a filter callback that encapsulates
/// all visibility rules (root selection, completed toggle, actionable check).
/// TTL consumes it for flattening and rendering.
#[derive(Clone, Copy)]
pub struct TaskTreeView {
    /// Shared indexed tree (from TaskStore).
    pub tree: Memo<TaskTree>,
    /// Filter callback — determines task visibility.
    pub filter: Signal<Callback<TaskModel, bool>>,
    /// Task IDs to keep visible regardless of filter (for undo/reassignment support).
    pub extra_visible: RwSignal<Vec<i64>>,
    /// Whether tasks have been loaded from server.
    pub is_loaded: Signal<bool>,
}

impl TaskTreeView {
    pub fn new(app_store: AppStore, filter: Signal<Callback<TaskModel, bool>>) -> Self {
        Self {
            tree: app_store.tasks.task_tree,
            filter,
            extra_visible: RwSignal::new(vec![]),
            is_loaded: app_store.tasks.loaded_signal(),
        }
    }
}
