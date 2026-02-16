use leptos::prelude::*;
use north_dto::Task;
use north_stores::AppStore;

#[derive(Clone, Copy)]
pub struct TaskListItemController {
    app_store: AppStore,
    task_id: i64,
    pub task: Memo<Option<Task>>,
}

impl TaskListItemController {
    pub fn new(app_store: AppStore, task_id: i64) -> Self {
        let task = app_store.tasks.get_by_id(task_id);
        Self {
            app_store,
            task_id,
            task,
        }
    }

    pub fn toggle_complete(&self) {
        let was_completed = self
            .task
            .get_untracked()
            .map(|t| t.completed_at.is_some())
            .unwrap_or(false);
        self.app_store
            .tasks
            .toggle_complete(self.task_id, was_completed);
    }

    pub fn delete(&self) {
        self.app_store.tasks.delete_task(self.task_id);
    }

    pub fn set_start_at(&self, start_at: String) {
        self.app_store.tasks.set_start_at(self.task_id, start_at);
    }

    pub fn clear_start_at(&self) {
        self.app_store.tasks.clear_start_at(self.task_id);
    }

    pub fn set_project(&self, project_id: i64) {
        self.app_store.tasks.set_project(self.task_id, project_id);
    }

    pub fn clear_project(&self) {
        self.app_store.tasks.clear_project(self.task_id);
    }

    pub fn set_tags(&self, tag_names: Vec<String>) {
        self.app_store.tasks.set_tags(self.task_id, tag_names);
    }

    pub fn review(&self) {
        self.app_store.tasks.review_task(self.task_id);
    }
}
