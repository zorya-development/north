use chrono::Utc;
use leptos::prelude::*;
use north_stores::{AppStore, IdFilter, TaskDetailModalStore, TaskStoreFilter};

#[derive(Clone, Copy)]
pub struct TodayController {
    task_detail_modal_store: TaskDetailModalStore,
    pub root_task_ids: Memo<Vec<i64>>,
    pub show_completed: RwSignal<bool>,
    pub completed_count: Memo<usize>,
    pub is_loaded: Signal<bool>,
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

        Self {
            task_detail_modal_store,
            root_task_ids,
            show_completed,
            completed_count,
            is_loaded,
        }
    }

    pub fn open_detail(&self, task_id: i64) {
        let task_ids = self.root_task_ids.get_untracked();
        self.task_detail_modal_store.open(task_id, task_ids);
    }
}
