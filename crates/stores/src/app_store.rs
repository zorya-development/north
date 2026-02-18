use crate::{
    FilterDslStore, ModalStore, ProjectStore, SavedFilterStore, StatusBarStore, TagStore,
    TaskCreateModalStore, TaskDetailModalStore, TaskStore,
};

#[derive(Clone, Copy)]
pub struct AppStore {
    pub tasks: TaskStore,
    pub projects: ProjectStore,
    pub tags: TagStore,
    pub saved_filters: SavedFilterStore,
    pub task_detail_modal: TaskDetailModalStore,
    pub task_create_modal: TaskCreateModalStore,
    pub filter_dsl: FilterDslStore,
    pub status_bar: StatusBarStore,
    pub modal: ModalStore,
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
        let saved_filters = SavedFilterStore::new();
        let modal = ModalStore::new();
        let task_detail_modal = TaskDetailModalStore::new(tasks, modal);
        let task_create_modal = TaskCreateModalStore::new(tasks, modal);
        let filter_dsl = FilterDslStore::new();
        let status_bar = StatusBarStore::new();

        Self {
            tasks,
            projects,
            tags,
            saved_filters,
            task_detail_modal,
            task_create_modal,
            filter_dsl,
            status_bar,
            modal,
        }
    }

    pub fn refetch(&self) {
        self.tasks.refetch();
        self.projects.refetch();
        self.tags.refetch();
        self.saved_filters.refetch();
    }
}
