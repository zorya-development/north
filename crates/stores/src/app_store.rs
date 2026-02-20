use crate::{
    BrowserStorageStore, FilterDslStore, ModalStore, ProjectStore, SavedFilterStore, SettingsStore,
    StatusBarStore, TagStore, TaskDetailModalStore, TaskStore,
};

#[derive(Clone, Copy)]
pub struct AppStore {
    pub tasks: TaskStore,
    pub projects: ProjectStore,
    pub tags: TagStore,
    pub saved_filters: SavedFilterStore,
    pub settings: SettingsStore,
    pub task_detail_modal: TaskDetailModalStore,
    pub filter_dsl: FilterDslStore,
    pub status_bar: StatusBarStore,
    pub modal: ModalStore,
    pub browser_storage: BrowserStorageStore,
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
        let settings = SettingsStore::new();
        let modal = ModalStore::new();
        let task_detail_modal = TaskDetailModalStore::new(tasks, modal);
        let filter_dsl = FilterDslStore::new();
        let status_bar = StatusBarStore::new();
        let browser_storage = BrowserStorageStore::new();

        Self {
            tasks,
            projects,
            tags,
            saved_filters,
            settings,
            task_detail_modal,
            filter_dsl,
            status_bar,
            modal,
            browser_storage,
        }
    }

    pub fn refetch(&self) {
        self.tasks.refetch();
        self.projects.refetch();
        self.tags.refetch();
        self.saved_filters.refetch();
        self.settings.refetch();
    }
}
