use leptos::prelude::*;
use north_stores::{AppStore, TaskModel};

/// Compute whether a task is actionable based on its parent's sequential_limit.
/// Root tasks are always actionable. Subtasks are actionable only if they are
/// within the first N incomplete siblings (sorted by sort_key) where N is the
/// parent's sequential_limit. A limit of 0 means unlimited.
pub fn is_actionable(task: &TaskModel, all_tasks: &[TaskModel]) -> bool {
    if task.someday {
        return false;
    }
    let Some(parent_id) = task.parent_id else {
        return true;
    };
    let limit = all_tasks
        .iter()
        .find(|t| t.id == parent_id)
        .map(|p| p.sequential_limit)
        .unwrap_or(1);
    if limit == 0 {
        return true;
    }
    let siblings_before = all_tasks
        .iter()
        .filter(|t| {
            t.parent_id == Some(parent_id) && t.completed_at.is_none() && t.sort_key < task.sort_key
        })
        .count();
    (siblings_before as i16) < limit
}

#[derive(Clone, Copy)]
pub struct ActionableController {
    pub count: Memo<usize>,
    pub hide: Signal<bool>,
    storage_key: &'static str,
    app_store: AppStore,
}

impl ActionableController {
    pub fn new(
        app_store: AppStore,
        all_tasks: Memo<Vec<TaskModel>>,
        storage_key: &'static str,
    ) -> Self {
        let hide = Signal::derive(move || app_store.browser_storage.get_bool(storage_key));

        let count = Memo::new(move |_| {
            let tasks = all_tasks.get();
            tasks
                .iter()
                .filter(|t| t.completed_at.is_none() && is_actionable(t, &tasks))
                .count()
        });

        Self {
            count,
            hide,
            storage_key,
            app_store,
        }
    }

    pub fn toggle(&self) {
        self.app_store.browser_storage.toggle_bool(self.storage_key);
    }
}
