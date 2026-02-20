use chrono::Utc;
use leptos::prelude::*;
use leptos::task::spawn_local;
use north_dto::ProjectStatus;
use north_repositories::TaskRepository;
use north_stores::{AppStore, IdFilter, TaskDetailModalStore, TaskStoreFilter};

const HIDE_NON_ACTIONABLE_KEY: &str = "north:hide-non-actionable:review";

#[derive(Clone, Copy)]
pub struct ReviewController {
    app_store: AppStore,
    task_detail_modal_store: TaskDetailModalStore,
    pub review_task_ids: Memo<Vec<i64>>,
    pub reviewed_task_ids: Memo<Vec<i64>>,
    pub is_loaded: Signal<bool>,
    pub show_reviewed: (ReadSignal<bool>, WriteSignal<bool>),
    pub hide_non_actionable: Signal<bool>,
    pub pending_filter: Callback<north_stores::TaskModel, bool>,
    pub reviewed_filter: Callback<north_stores::TaskModel, bool>,
}

impl ReviewController {
    pub fn new(app_store: AppStore) -> Self {
        let task_detail_modal_store = app_store.task_detail_modal;
        let show_reviewed = signal(false);

        let review_interval = app_store.settings.review_interval_days();

        // All active top-level tasks
        let all_active = app_store.tasks.filtered(TaskStoreFilter {
            project_id: IdFilter::Any,
            parent_id: IdFilter::IsNull,
            is_completed: Some(false),
        });

        let active_projects = app_store.projects;

        // Tasks due for review: reviewed_at is null or older than cutoff
        let review_task_ids = Memo::new(move |_| {
            let interval = review_interval.get();
            let cutoff = Utc::now().date_naive() - chrono::Duration::days(interval);
            let projects = active_projects.get();

            all_active
                .get()
                .into_iter()
                .filter(|t| {
                    // Only include tasks in active projects or no project
                    if let Some(pid) = t.project_id {
                        projects
                            .iter()
                            .find(|p| p.id == pid)
                            .is_some_and(|p| p.status == ProjectStatus::Active)
                    } else {
                        true
                    }
                })
                .filter(|t| match t.reviewed_at {
                    None => true,
                    Some(date) => date <= cutoff,
                })
                .map(|t| t.id)
                .collect()
        });

        // Recently reviewed tasks: reviewed_at is set and newer than cutoff
        let reviewed_task_ids = Memo::new(move |_| {
            if !show_reviewed.0.get() {
                return vec![];
            }
            let interval = review_interval.get();
            let cutoff = Utc::now().date_naive() - chrono::Duration::days(interval);
            let projects = active_projects.get();

            all_active
                .get()
                .into_iter()
                .filter(|t| {
                    if let Some(pid) = t.project_id {
                        projects
                            .iter()
                            .find(|p| p.id == pid)
                            .is_some_and(|p| p.status == ProjectStatus::Active)
                    } else {
                        true
                    }
                })
                .filter(|t| match t.reviewed_at {
                    Some(date) => date > cutoff,
                    None => false,
                })
                .map(|t| t.id)
                .collect()
        });

        let is_loaded = app_store.tasks.loaded_signal();

        let hide_non_actionable =
            Signal::derive(move || app_store.browser_storage.get_bool(HIDE_NON_ACTIONABLE_KEY));

        let show_completed = RwSignal::new(false);
        let show_completed_reviewed = RwSignal::new(false);

        let pending_filter = Callback::new(move |task: north_stores::TaskModel| {
            if task.completed_at.is_some() {
                show_completed.get()
            } else {
                !hide_non_actionable.get() || task.actionable
            }
        });

        let reviewed_filter = Callback::new(move |task: north_stores::TaskModel| {
            task.completed_at.is_none() || show_completed_reviewed.get()
        });

        Self {
            app_store,
            task_detail_modal_store,
            review_task_ids,
            reviewed_task_ids,
            is_loaded,
            show_reviewed,
            hide_non_actionable,
            pending_filter,
            reviewed_filter,
        }
    }

    pub fn review_all(&self) {
        let app_store = self.app_store;
        spawn_local(async move {
            if TaskRepository::review_all().await.is_ok() {
                app_store.tasks.refetch();
            }
        });
    }

    pub fn open_detail(&self, task_id: i64) {
        let task_ids = self.review_task_ids.get_untracked();
        self.task_detail_modal_store.open(task_id, task_ids);
    }

    pub fn toggle_actionable_visibility(&self) {
        self.app_store
            .browser_storage
            .toggle_bool(HIDE_NON_ACTIONABLE_KEY);
    }
}
