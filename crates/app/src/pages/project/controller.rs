use leptos::prelude::*;
use north_dto::Project;
use north_stores::{AppStore, IdFilter, TaskDetailModalStore, TaskStoreFilter};

#[derive(Clone, Copy)]
pub struct ProjectController {
    task_detail_modal_store: TaskDetailModalStore,
    pub project: Memo<Option<Project>>,
    pub root_task_ids: Memo<Vec<i64>>,
    pub show_completed: RwSignal<bool>,
    pub completed_count: Memo<usize>,
    pub is_loaded: Signal<bool>,
    app_store: AppStore,
}

impl ProjectController {
    pub fn new(app_store: AppStore, project_id: Signal<i64>) -> Self {
        let task_detail_modal_store = app_store.task_detail_modal;

        Effect::new(move |_| {
            app_store.tasks.refetch();
        });

        let project = Memo::new(move |_| {
            let pid = project_id.get();
            app_store.projects.get().into_iter().find(|p| p.id == pid)
        });

        let root_tasks = Memo::new(move |_| {
            let pid = project_id.get();
            app_store
                .tasks
                .filtered(TaskStoreFilter {
                    project_id: IdFilter::Is(pid),
                    parent_id: IdFilter::IsNull,
                    is_completed: None,
                })
                .get()
        });

        let root_task_ids = Memo::new(move |_| root_tasks.get().iter().map(|t| t.id).collect());

        let completed_tasks = Memo::new(move |_| {
            let pid = project_id.get();
            app_store
                .tasks
                .filtered(TaskStoreFilter {
                    project_id: IdFilter::Is(pid),
                    parent_id: IdFilter::IsNull,
                    is_completed: Some(true),
                })
                .get()
        });

        let completed_count = Memo::new(move |_| completed_tasks.get().len());

        let show_completed = RwSignal::new(false);
        let is_loaded = app_store.tasks.loaded_signal();

        Self {
            task_detail_modal_store,
            project,
            root_task_ids,
            show_completed,
            completed_count,
            is_loaded,
            app_store,
        }
    }

    pub fn open_detail(&self, task_id: i64) {
        let task_ids = self.root_task_ids.get_untracked();
        self.task_detail_modal_store.open(task_id, task_ids);
    }

    pub fn reorder_task(&self, task_id: i64, sort_key: String, parent_id: Option<Option<i64>>) {
        self.app_store
            .tasks
            .reorder_task(task_id, sort_key, parent_id);
    }
}
