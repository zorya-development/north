use leptos::prelude::*;
use north_domain::Task;
use north_stores::{AppStore, IdFilter, TaskDetailModalStore, TaskStoreFilter};

#[derive(Clone, Copy)]
pub struct InboxController {
    app_store: AppStore,
    task_detail_modal_store: TaskDetailModalStore,
    pub active_task_ids: Memo<Vec<i64>>,
    pub completed_task_ids: Memo<Vec<i64>>,
    pub completed_count: Memo<usize>,
    pub is_loaded: Signal<bool>,
    pub is_new_task_form_open: (ReadSignal<bool>, WriteSignal<bool>),
    pub active_tasks_for_reorder: Memo<Vec<Task>>,
}

impl InboxController {
    pub fn new(app_store: AppStore) -> Self {
        let task_detail_modal_store = app_store.task_detail_modal;

        Effect::new(move |_| {
            app_store.tasks.refetch();
        });

        let active_tasks = app_store.tasks.filtered(TaskStoreFilter {
            project_id: IdFilter::IsNull,
            parent_id: IdFilter::IsNull,
            is_completed: Some(false),
        });

        let completed_tasks = app_store.tasks.filtered(TaskStoreFilter {
            project_id: IdFilter::IsNull,
            parent_id: IdFilter::IsNull,
            is_completed: Some(true),
        });

        let active_task_ids =
            Memo::new(move |_| active_tasks.get().iter().map(|t| t.id).collect());

        let completed_task_ids =
            Memo::new(move |_| completed_tasks.get().iter().map(|t| t.id).collect());

        let completed_count = Memo::new(move |_| completed_tasks.get().len());

        let is_loaded = app_store.tasks.loaded_signal();

        let is_new_task_form_open = signal(false);

        let active_tasks_for_reorder = app_store.tasks.filtered(TaskStoreFilter {
            project_id: IdFilter::IsNull,
            parent_id: IdFilter::IsNull,
            is_completed: Some(false),
        });

        Self {
            app_store,
            task_detail_modal_store,
            active_task_ids,
            completed_task_ids,
            completed_count,
            is_loaded,
            is_new_task_form_open,
            active_tasks_for_reorder,
        }
    }

    pub fn open_detail(&self, task_id: i64) {
        let task_ids = self.active_task_ids.get_untracked();
        self.task_detail_modal_store.open(task_id, task_ids);
    }

    pub fn reorder_task(&self, task_id: i64, sort_key: String, parent_id: Option<Option<i64>>) {
        self.app_store
            .tasks
            .reorder_task(task_id, sort_key, parent_id);
    }
}
