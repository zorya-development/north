use chrono::Utc;
use leptos::prelude::*;
use north_stores::{AppStore, IdFilter, TaskDetailModalStore, TaskModel, TaskStoreFilter};

use crate::containers::traversable_task_list::{ActionableToggle, CompletedToggle, ToolbarConfig};
use crate::libs::{is_actionable, KeepCompletedVisible};

const HIDE_NON_ACTIONABLE_KEY: &str = "north:hide-non-actionable:today";

#[derive(Clone, Copy)]
pub struct TodayController {
    task_detail_modal_store: TaskDetailModalStore,
    pub root_task_ids: Memo<Vec<i64>>,
    pub show_completed: RwSignal<bool>,
    pub completed_count: Memo<usize>,
    pub is_loaded: Signal<bool>,
    pub hide_non_actionable: Signal<bool>,
    pub actionable_count: Memo<usize>,
    pub node_filter: Signal<Callback<north_stores::TaskModel, bool>>,
    app_store: AppStore,
}

impl TodayController {
    pub fn new(app_store: AppStore) -> Self {
        let task_detail_modal_store = app_store.task_detail_modal;

        Effect::new(move |_| {
            app_store.tasks.refetch();
        });

        // All top-level tasks — then post-filter by start_at <= now
        let base_all = app_store.tasks.filtered(TaskStoreFilter {
            project_id: IdFilter::Any,
            parent_id: IdFilter::IsNull,
            is_completed: None,
            ..Default::default()
        });

        let root_task_ids = Memo::new(move |_| {
            let now = Utc::now();
            base_all
                .get()
                .into_iter()
                .filter(|t| t.start_at.map(|dt| dt <= now).unwrap_or(false))
                .map(|t| t.id)
                .collect()
        });

        // Completed count for toggle
        let base_completed = app_store.tasks.filtered(TaskStoreFilter {
            project_id: IdFilter::Any,
            parent_id: IdFilter::IsNull,
            is_completed: Some(true),
            ..Default::default()
        });

        let completed_count = Memo::new(move |_| {
            let now = Utc::now();
            base_completed
                .get()
                .into_iter()
                .filter(|t| t.start_at.map(|dt| dt <= now).unwrap_or(false))
                .count()
        });

        let show_completed = RwSignal::new(false);
        let is_loaded = app_store.tasks.loaded_signal();

        let keep_completed = KeepCompletedVisible::new();
        provide_context(keep_completed);

        let hide_non_actionable =
            Signal::derive(move || app_store.browser_storage.get_bool(HIDE_NON_ACTIONABLE_KEY));

        let all_tasks = app_store.tasks.filtered(TaskStoreFilter::default());

        let actionable_count = Memo::new(move |_| {
            let tasks = all_tasks.get();
            tasks
                .iter()
                .filter(|t| t.completed_at.is_none() && is_actionable(t, &tasks))
                .count()
        });

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
            actionable_count,
            node_filter,
            app_store,
        }
    }

    pub fn open_detail(&self, task_id: i64) {
        let task_ids = self.root_task_ids.get_untracked();
        self.task_detail_modal_store.open(task_id, task_ids);
    }

    pub fn toolbar_config(&self) -> ToolbarConfig {
        let show_completed = self.show_completed;
        let completed_count = self.completed_count;
        let hide_non_actionable = self.hide_non_actionable;
        let actionable_count = self.actionable_count;
        let app_store = self.app_store;
        ToolbarConfig {
            enabled: true,
            show_add_task: true,
            completed: Some(CompletedToggle {
                is_active: show_completed.into(),
                count: completed_count,
                on_toggle: Callback::new(move |()| {
                    show_completed.update(|v| *v = !*v);
                }),
            }),
            actionable: Some(ActionableToggle {
                is_active: hide_non_actionable,
                count: actionable_count,
                on_toggle: Callback::new(move |()| {
                    app_store
                        .browser_storage
                        .toggle_bool(HIDE_NON_ACTIONABLE_KEY);
                }),
            }),
        }
    }
}
