use leptos::prelude::*;
use north_dto::Project;
use north_stores::{AppStore, TaskModel};

#[derive(Clone, Copy)]
pub struct TaskListItemController {
    app_store: AppStore,
    task_id: i64,
    pub task: Memo<Option<TaskModel>>,
    pub projects: Signal<Vec<Project>>,
}

impl TaskListItemController {
    pub fn new(app_store: AppStore, task_id: i64) -> Self {
        let task = app_store.tasks.get_by_id(task_id);
        let projects = Signal::derive(move || app_store.projects.get());
        Self {
            app_store,
            task_id,
            task,
            projects,
        }
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
