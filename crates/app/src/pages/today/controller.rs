use std::collections::BTreeMap;

use chrono::Utc;
use leptos::prelude::*;
use north_domain::Task;
use north_stores::{AppStore, IdFilter, TaskDetailModalStore, TaskStoreFilter};

/// (project_title or "No Project", task_ids in that group)
pub type GroupedTasks = Vec<(String, Vec<i64>)>;

#[derive(Clone, Copy)]
pub struct TodayController {
    app_store: AppStore,
    task_detail_modal_store: TaskDetailModalStore,
    pub grouped_task_ids: Memo<GroupedTasks>,
    pub all_active_task_ids: Memo<Vec<i64>>,
    pub completed_task_ids: Memo<Vec<i64>>,
    pub completed_count: Memo<usize>,
    pub is_loaded: Signal<bool>,
    pub is_new_task_form_open: (ReadSignal<bool>, WriteSignal<bool>),
    pub active_tasks_for_reorder: Memo<Vec<Task>>,
}

impl TodayController {
    pub fn new(app_store: AppStore) -> Self {
        let task_detail_modal_store = app_store.task_detail_modal;

        Effect::new(move |_| {
            app_store.tasks.refetch();
        });

        // All top-level incomplete tasks — then post-filter by start_at <= now
        let base_active = app_store.tasks.filtered(TaskStoreFilter {
            project_id: IdFilter::Any,
            parent_id: IdFilter::IsNull,
            is_completed: Some(false),
        });

        let today_active = Memo::new(move |_| {
            let now = Utc::now();
            base_active
                .get()
                .into_iter()
                .filter(|t| t.start_at.map(|dt| dt <= now).unwrap_or(false))
                .collect::<Vec<_>>()
        });

        let grouped_task_ids = Memo::new(move |_| {
            let tasks = today_active.get();
            let mut groups: BTreeMap<String, Vec<i64>> = BTreeMap::new();
            for task in &tasks {
                let label = task
                    .project_title
                    .clone()
                    .unwrap_or_else(|| "No Project".to_string());
                groups.entry(label).or_default().push(task.id);
            }
            groups.into_iter().collect()
        });

        let all_active_task_ids =
            Memo::new(move |_| today_active.get().iter().map(|t| t.id).collect());

        let active_tasks_for_reorder = Memo::new(move |_| today_active.get());

        // Completed tasks with start_at <= now
        let base_completed = app_store.tasks.filtered(TaskStoreFilter {
            project_id: IdFilter::Any,
            parent_id: IdFilter::IsNull,
            is_completed: Some(true),
        });

        let today_completed = Memo::new(move |_| {
            let now = Utc::now();
            base_completed
                .get()
                .into_iter()
                .filter(|t| t.start_at.map(|dt| dt <= now).unwrap_or(false))
                .collect::<Vec<_>>()
        });

        let completed_task_ids =
            Memo::new(move |_| today_completed.get().iter().map(|t| t.id).collect());

        let completed_count = Memo::new(move |_| today_completed.get().len());

        let is_loaded = app_store.tasks.loaded_signal();
        let is_new_task_form_open = signal(false);

        Self {
            app_store,
            task_detail_modal_store,
            grouped_task_ids,
            all_active_task_ids,
            completed_task_ids,
            completed_count,
            is_loaded,
            is_new_task_form_open,
            active_tasks_for_reorder,
        }
    }

    pub fn open_detail(&self, task_id: i64) {
        let task_ids = self.all_active_task_ids.get_untracked();
        self.task_detail_modal_store.open(task_id, task_ids);
    }

    pub fn reorder_task(&self, task_id: i64, sort_key: String, parent_id: Option<Option<i64>>) {
        self.app_store
            .tasks
            .reorder_task(task_id, sort_key, parent_id);
    }
}
