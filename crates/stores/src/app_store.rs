use crate::{ProjectStore, TaskDetailModalStore, TaskStore};

#[derive(Clone, Copy)]
pub struct AppStore {
    pub tasks: TaskStore,
    pub projects: ProjectStore,
    pub task_detail_modal: TaskDetailModalStore,
}

impl Default for AppStore {
    fn default() -> Self {
        Self::new()
    }
}

impl AppStore {
    pub fn new() -> Self {
        let tasks = TaskStore::new();
        let projects = ProjectStore::new();
        let task_detail_modal = TaskDetailModalStore::new(tasks);

        Self {
            tasks,
            projects,
            task_detail_modal,
        }
    }
}
