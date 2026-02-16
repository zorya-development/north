use leptos::prelude::*;
use north_dto::CreateTask;

use crate::TaskStore;

#[derive(Clone, Copy)]
pub struct TaskCreateModalStore {
    task_store: TaskStore,
    is_open: RwSignal<bool>,
    default_project_id: RwSignal<Option<i64>>,
    default_parent_id: RwSignal<Option<i64>>,
}

impl TaskCreateModalStore {
    pub fn new(task_store: TaskStore) -> Self {
        Self {
            task_store,
            is_open: RwSignal::new(false),
            default_project_id: RwSignal::new(None),
            default_parent_id: RwSignal::new(None),
        }
    }

    pub fn open(&self, project_id: Option<i64>, parent_id: Option<i64>) {
        self.default_project_id.set(project_id);
        self.default_parent_id.set(parent_id);
        self.is_open.set(true);
    }

    pub fn close(&self) {
        self.is_open.set(false);
        self.default_project_id.set(None);
        self.default_parent_id.set(None);
    }

    pub fn is_open(&self) -> bool {
        self.is_open.get()
    }

    pub fn is_open_signal(&self) -> RwSignal<bool> {
        self.is_open
    }

    pub fn default_project_id(&self) -> Option<i64> {
        self.default_project_id.get()
    }

    pub fn default_parent_id(&self) -> Option<i64> {
        self.default_parent_id.get()
    }

    pub fn create_task(&self, input: CreateTask) {
        self.task_store.create_task(input);
        self.close();
    }
}
