use crate::{ProjectStore, TagStore, TaskDetailModalStore, TaskStore};

#[derive(Clone, Copy)]
pub struct AppStore {
    pub tasks: TaskStore,
    pub projects: ProjectStore,
    pub tags: TagStore,
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
        let tags = TagStore::new();
        let task_detail_modal = TaskDetailModalStore::new(tasks);

        Self {
            tasks,
            projects,
            tags,
            task_detail_modal,
        }
    }
}
