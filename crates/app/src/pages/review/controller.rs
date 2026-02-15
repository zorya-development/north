use chrono::Utc;
use leptos::prelude::*;
use leptos::task::spawn_local;
use north_domain::ProjectStatus;
use north_repositories::{SettingsRepository, TaskRepository};
use north_stores::{AppStore, IdFilter, TaskDetailModalStore, TaskStoreFilter};

#[derive(Clone, Copy)]
pub struct ReviewController {
    app_store: AppStore,
    task_detail_modal_store: TaskDetailModalStore,
    pub review_task_ids: Memo<Vec<i64>>,
    pub reviewed_task_ids: Memo<Vec<i64>>,
    pub is_loaded: Signal<bool>,
    pub show_reviewed: (ReadSignal<bool>, WriteSignal<bool>),
}

impl ReviewController {
    pub fn new(app_store: AppStore) -> Self {
        let task_detail_modal_store = app_store.task_detail_modal;
        let show_reviewed = signal(false);

        // Load review interval from user settings
        let review_interval = RwSignal::new(7_i64);
        spawn_local(async move {
            if let Ok(settings) = SettingsRepository::get().await {
                review_interval.set(settings.review_interval_days as i64);
            }
        });

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

        Self {
            app_store,
            task_detail_modal_store,
            review_task_ids,
            reviewed_task_ids,
            is_loaded,
            show_reviewed,
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
}
