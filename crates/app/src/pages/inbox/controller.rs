use leptos::prelude::*;
use north_stores::{AppStore, IdFilter, TaskStoreFilter};

#[derive(Clone, Copy)]
pub struct InboxController {
    app_store: AppStore,
    pub active_task_ids: Memo<Vec<i64>>,
    pub completed_task_ids: Memo<Vec<i64>>,
    pub completed_count: Memo<usize>,
    pub is_new_task_form_open: (ReadSignal<bool>, WriteSignal<bool>),
}

impl InboxController {
    pub fn new(app_store: AppStore) -> Self {
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
            Memo::new(move |_| active_tasks.get().iter().map(|t| t.task.id).collect());

        let completed_task_ids =
            Memo::new(move |_| completed_tasks.get().iter().map(|t| t.task.id).collect());

        let completed_count = Memo::new(move |_| completed_tasks.get().len());

        let is_new_task_form_open = signal(false);

        Self {
            app_store,
            active_task_ids,
            completed_task_ids,
            completed_count,
            is_new_task_form_open,
        }
    }

    pub fn active_tasks_for_reorder(&self) -> Memo<Vec<north_domain::TaskWithMeta>> {
        self.app_store.tasks.filtered(TaskStoreFilter {
            project_id: IdFilter::IsNull,
            parent_id: IdFilter::IsNull,
            is_completed: Some(false),
        })
    }

    pub fn reorder_task(&self, task_id: i64, sort_key: String, parent_id: Option<Option<i64>>) {
        self.app_store
            .tasks
            .reorder_task(task_id, sort_key, parent_id);
    }
}
