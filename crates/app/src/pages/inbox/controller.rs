use leptos::prelude::*;
use north_stores::{AppStore, IdFilter, TaskDetailModalStore, TaskStoreFilter};

#[derive(Clone, Copy)]
pub struct InboxController {
    app_store: AppStore,
    task_detail_modal_store: TaskDetailModalStore,
    pub root_task_ids: Memo<Vec<i64>>,
    pub show_completed: RwSignal<bool>,
    pub completed_count: Memo<usize>,
    pub is_loaded: Signal<bool>,
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

        let root_task_ids = Memo::new(move |_| root_tasks.get().iter().map(|t| t.id).collect());

        let completed_tasks = app_store.tasks.filtered(TaskStoreFilter {
            project_id: IdFilter::IsNull,
            parent_id: IdFilter::IsNull,
            is_completed: Some(true),
        });

        let completed_count = Memo::new(move |_| completed_tasks.get().len());

        let show_completed = RwSignal::new(false);
        let is_loaded = app_store.tasks.loaded_signal();

        Self {
            app_store,
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

    pub fn open_create(&self) {
        self.app_store.task_create_modal.open(None, None);
    }

    pub fn reorder_task(&self, task_id: i64, sort_key: String, parent_id: Option<Option<i64>>) {
        self.app_store
            .tasks
            .reorder_task(task_id, sort_key, parent_id);
    }
}
