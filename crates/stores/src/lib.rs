pub mod task_store;

pub use task_store::{IdFilter, TaskStore, TaskStoreFilter};

use leptos::prelude::*;

#[derive(Clone, Copy)]
pub struct AppStore {
    pub tasks: TaskStore,
}

impl AppStore {
    pub fn new() -> Self {
        Self {
            tasks: TaskStore::new(),
        }
    }

    pub fn provide(self) {
        provide_context(self);
    }
}

pub fn use_app_store() -> AppStore {
    expect_context::<AppStore>()
}
