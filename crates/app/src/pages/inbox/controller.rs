use leptos::prelude::*;
use north_domain::{CreateTask, TaskWithMeta};
use north_stores::{AppStore, IdFilter, TaskStoreFilter};

#[derive(Clone, Copy)]
pub struct InboxController {
    app_store: AppStore,
    pub tasks: Memo<Vec<TaskWithMeta>>,
}

impl InboxController {
    pub fn new(app_store: AppStore) -> Self {
        let tasks = app_store.tasks.filtered(TaskStoreFilter {
            project_id: IdFilter::IsNull,
            parent_id: IdFilter::IsNull,
            ..Default::default()
        });
        Self { app_store, tasks }
    }

    pub fn toggle_complete(&self, id: i64, was_completed: bool) {
        self.app_store.tasks.toggle_complete(id, was_completed);
    }

    pub fn delete_task(&self, id: i64) {
        self.app_store.tasks.delete_task(id);
    }

    pub fn update_task(&self, id: i64, title: String, body: Option<String>) {
        self.app_store.tasks.update_task(id, title, body);
    }

    pub fn create_task(&self, title: String, body: Option<String>) {
        self.app_store.tasks.create_task(CreateTask {
            title,
            body,
            ..Default::default()
        });
    }

    pub fn set_start_at(&self, id: i64, start_at: String) {
        self.app_store.tasks.set_start_at(id, start_at);
    }

    pub fn clear_start_at(&self, id: i64) {
        self.app_store.tasks.clear_start_at(id);
    }

    pub fn set_project(&self, task_id: i64, project_id: i64) {
        self.app_store.tasks.set_project(task_id, project_id);
    }

    pub fn clear_project(&self, task_id: i64) {
        self.app_store.tasks.clear_project(task_id);
    }

    pub fn set_tags(&self, task_id: i64, tag_names: Vec<String>) {
        self.app_store.tasks.set_tags(task_id, tag_names);
    }

    pub fn review_task(&self, id: i64) {
        self.app_store.tasks.review_task(id);
    }

    pub fn reorder_task(&self, task_id: i64, sort_key: String, parent_id: Option<Option<i64>>) {
        self.app_store
            .tasks
            .reorder_task(task_id, sort_key, parent_id);
    }
}
