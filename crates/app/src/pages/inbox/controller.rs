use leptos::prelude::*;
use north_stores::{AppStore, IdFilter, TaskDetailModalStore, TaskModel, TaskStoreFilter};

use crate::libs::{is_actionable, KeepCompletedVisible, KeepTaskVisible};

const HIDE_NON_ACTIONABLE_KEY: &str = "north:hide-non-actionable:inbox";

#[derive(Clone, Copy)]
pub struct InboxController {
    task_detail_modal_store: TaskDetailModalStore,
    pub root_task_ids: Memo<Vec<i64>>,
    pub show_completed: RwSignal<bool>,
    pub completed_count: Memo<usize>,
    pub is_loaded: Signal<bool>,
    pub hide_non_actionable: Signal<bool>,
    pub node_filter: Signal<Callback<north_stores::TaskModel, bool>>,
    app_store: AppStore,
}

impl InboxController {
    pub fn new(app_store: AppStore) -> Self {
        let task_detail_modal_store = app_store.task_detail_modal;

        Effect::new(move |_| {
            app_store.tasks.refetch();
        });

        // All root inbox tasks (active + completed) for tree building.
        let root_tasks = app_store.tasks.filtered(TaskStoreFilter {
            project_id: IdFilter::IsNull,
            parent_id: IdFilter::IsNull,
            is_completed: None,
        });

        // IDs of tasks that should stay visible even if they no longer match
        // the inbox filter (e.g. assigned to a project via @token or detail modal).
        let extra_show_ids: RwSignal<Vec<i64>> = RwSignal::new(vec![]);
        provide_context(KeepTaskVisible::new(extra_show_ids));

        let keep_completed = KeepCompletedVisible::new();
        provide_context(keep_completed);

        // Track tasks that disappear from the base filter (updated via detail
        // modal, action bar pickers, etc.) and keep them visible.
        let prev_filtered_ids: RwSignal<Vec<i64>> = RwSignal::new(vec![]);
        Effect::new(move |_| {
            let current: Vec<i64> = root_tasks.get().iter().map(|t| t.id).collect();
            let prev = prev_filtered_ids.get_untracked();
            for &id in &prev {
                if !current.contains(&id) {
                    extra_show_ids.update(|ids| {
                        if !ids.contains(&id) {
                            ids.push(id);
                        }
                    });
                }
            }
            prev_filtered_ids.set(current);
        });

        let all_root_tasks = app_store.tasks.filtered(TaskStoreFilter {
            project_id: IdFilter::Any,
            parent_id: IdFilter::IsNull,
            is_completed: None,
        });

        let root_task_ids = Memo::new(move |_| {
            let mut ids: Vec<i64> = root_tasks.get().iter().map(|t| t.id).collect();
            let extras = extra_show_ids.get();
            if !extras.is_empty() {
                let all = all_root_tasks.get();
                for &eid in &extras {
                    if !ids.contains(&eid) && all.iter().any(|t| t.id == eid) {
                        ids.push(eid);
                    }
                }
            }
            ids
        });

        let completed_tasks = app_store.tasks.filtered(TaskStoreFilter {
            project_id: IdFilter::IsNull,
            parent_id: IdFilter::IsNull,
            is_completed: Some(true),
        });

        let completed_count = Memo::new(move |_| completed_tasks.get().len());

        let show_completed = RwSignal::new(false);
        let is_loaded = app_store.tasks.loaded_signal();

        let hide_non_actionable =
            Signal::derive(move || app_store.browser_storage.get_bool(HIDE_NON_ACTIONABLE_KEY));

        let all_tasks = app_store.tasks.filtered(TaskStoreFilter::default());

        let keep_completed_signal = keep_completed.signal();
        let node_filter = Signal::derive(move || {
            let hide = hide_non_actionable.get();
            let show = show_completed.get();
            let pinned = keep_completed_signal.get();
            Callback::new(move |task: TaskModel| {
                if task.completed_at.is_some() {
                    return show || pinned.contains(&task.id);
                }
                if !hide {
                    return true;
                }
                is_actionable(&task, &all_tasks.get_untracked())
            })
        });

        Self {
            task_detail_modal_store,
            root_task_ids,
            show_completed,
            completed_count,
            is_loaded,
            hide_non_actionable,
            node_filter,
            app_store,
        }
    }

    pub fn open_detail(&self, task_id: i64) {
        let task_ids = self.root_task_ids.get_untracked();
        self.task_detail_modal_store.open(task_id, task_ids);
    }

    pub fn reorder_task(&self, task_id: i64, sort_key: String, parent_id: Option<Option<i64>>) {
        self.app_store
            .tasks
            .reorder_task(task_id, sort_key, parent_id);
    }

    pub fn toggle_actionable_visibility(&self) {
        self.app_store
            .browser_storage
            .toggle_bool(HIDE_NON_ACTIONABLE_KEY);
    }
}
