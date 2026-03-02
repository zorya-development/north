use chrono::Utc;
use leptos::prelude::*;
use north_dto::ProjectStatus;
use north_stores::{AppStore, IdFilter, TaskDetailModalStore, TaskStoreFilter};

use crate::containers::traversable_task_list::ToolbarConfig;
use crate::controllers::{build_toolbar, ActionableController, NodeFilterController};

const HIDE_NON_ACTIONABLE_KEY: &str = "north:hide-non-actionable:review";

#[derive(Clone, Copy)]
pub struct ReviewController {
    task_detail_modal_store: TaskDetailModalStore,
    pub review_task_ids: Memo<Vec<i64>>,
    pub reviewed_task_ids: Memo<Vec<i64>>,
    pub is_loaded: Signal<bool>,
    pub show_reviewed: (ReadSignal<bool>, WriteSignal<bool>),
    pub pending_filter: Signal<Callback<north_stores::TaskModel, bool>>,
    pub reviewed_filter: Signal<Callback<north_stores::TaskModel, bool>>,
    actionable: ActionableController,
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
            ..Default::default()
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
                .filter(|t| !t.someday)
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
                .filter(|t| !t.someday)
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

        let all_tasks = app_store.tasks.filtered(TaskStoreFilter::default());
        let actionable = ActionableController::new(app_store, all_tasks, HIDE_NON_ACTIONABLE_KEY);

        // Pending filter uses NodeFilterController (no completed toggle)
        let nf = NodeFilterController::new(&actionable, None, all_tasks);
        let pending_filter = nf.node_filter;

        // Reviewed filter: simple completed check + keep-completed pinning
        let keep_completed_signal = nf.keep_completed.signal();
        let reviewed_filter = Signal::derive(move || {
            let pinned = keep_completed_signal.get();
            Callback::new(move |task: north_stores::TaskModel| {
                task.completed_at.is_none() || pinned.contains(&task.id)
            })
        });

        Self {
            task_detail_modal_store,
            review_task_ids,
            reviewed_task_ids,
            is_loaded: app_store.tasks.loaded_signal(),
            show_reviewed,
            pending_filter,
            reviewed_filter,
            actionable,
        }
    }

    pub fn open_detail(&self, task_id: i64) {
        let task_ids = self.review_task_ids.get_untracked();
        self.task_detail_modal_store.open(task_id, task_ids);
    }

    pub fn toolbar_config(&self) -> ToolbarConfig {
        build_toolbar(false, None, &self.actionable)
    }
}
