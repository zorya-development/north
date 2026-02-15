use crate::{ProjectStore, TaskStore};

#[derive(Clone, Copy, Default)]
pub struct AppStore {
    pub tasks: TaskStore,
    pub projects: ProjectStore,
}

impl AppStore {
    pub fn new() -> Self {
        Self {
            tasks: TaskStore::new(),
            projects: ProjectStore::new(),
        }
    }
}
