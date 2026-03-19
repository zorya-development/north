use chrono::Utc;
use leptos::prelude::*;
use north_dto::ProjectStatus;
use north_stores::{AppStore, TaskDetailModalStore, TaskModel};

use crate::containers::traversable_task_list::{ActionableToggle, ToolbarConfig};
use crate::controllers::TaskTreeView;
use crate::libs::{KeepCompletedVisible, KeepTaskVisible};

const HIDE_NON_ACTIONABLE_KEY: &str = "north:hide-non-actionable:review";

#[derive(Clone, Copy)]
pub struct ReviewController {
    task_detail_modal_store: TaskDetailModalStore,
    pub pending_view: TaskTreeView,
    pub reviewed_view: TaskTreeView,
    pub show_reviewed: (ReadSignal<bool>, WriteSignal<bool>),
    pub actionable_count: Memo<usize>,
    pub hide_actionable: Signal<bool>,
    app_store: AppStore,
}

impl ReviewController {
    pub fn new(app_store: AppStore) -> Self {
        let show_reviewed = signal(false);
        let review_interval = app_store.settings.review_interval_days();
        let active_projects = app_store.projects;
        let tree = app_store.tasks.task_tree;

        let hide_actionable =
            Signal::derive(move || app_store.browser_storage.get_bool(HIDE_NON_ACTIONABLE_KEY));

        let keep_completed = KeepCompletedVisible::new();
        provide_context(keep_completed);
        let keep_completed_signal = keep_completed.signal();

        // Helper: check if a root task is eligible for review
        // (not someday, in active project or no project)
        let is_review_eligible = move |task: &TaskModel, projects: &[north_dto::Project]| {
            if task.someday {
                return false;
            }
            if let Some(pid) = task.project_id {
                projects
                    .iter()
                    .find(|p| p.id == pid)
                    .is_some_and(|p| p.status == ProjectStatus::Active)
            } else {
                true
            }
        };

        // Pending review filter: needs review + actionable check
        let pending_filter = Signal::derive(move || {
            let interval = review_interval.get();
            let cutoff = Utc::now().date_naive() - chrono::Duration::days(interval);
            let projects = active_projects.get();
            let hide = hide_actionable.get();
            let tree = tree.get();
            let pinned = keep_completed_signal.get();
            Callback::new(move |task: TaskModel| {
                if task.completed_at.is_some() {
                    return pinned.contains(&task.id);
                }
                // Root gating: eligible + needs review
                if task.parent_id.is_none() {
                    if !is_review_eligible(&task, &projects) {
                        return false;
                    }
                    let needs_review = match task.reviewed_at {
                        None => true,
                        Some(date) => date <= cutoff,
                    };
                    if !needs_review {
                        return false;
                    }
                }
                if hide && !tree.is_actionable(task.id) {
                    return false;
                }
                true
            })
        });

        let pending_view = TaskTreeView::new(app_store, pending_filter);
        provide_context(KeepTaskVisible::new(pending_view.extra_visible));

        // Reviewed filter: recently reviewed, no actionable check
        let show_reviewed_read = show_reviewed.0;
        let reviewed_filter = Signal::derive(move || {
            let show = show_reviewed_read.get();
            let interval = review_interval.get();
            let cutoff = Utc::now().date_naive() - chrono::Duration::days(interval);
            let projects = active_projects.get();
            let pinned = keep_completed_signal.get();
            Callback::new(move |task: TaskModel| {
                if !show {
                    return false;
                }
                if task.completed_at.is_some() {
                    return pinned.contains(&task.id);
                }
                // Root gating: eligible + recently reviewed
                if task.parent_id.is_none() {
                    if !is_review_eligible(&task, &projects) {
                        return false;
                    }
                    let recently_reviewed = match task.reviewed_at {
                        Some(date) => date > cutoff,
                        None => false,
                    };
                    if !recently_reviewed {
                        return false;
                    }
                }
                true
            })
        });

        let reviewed_view = TaskTreeView::new(app_store, reviewed_filter);

        let actionable_count = Memo::new(move |_| {
            let tree = tree.get();
            tree.count_matching(|t| t.completed_at.is_none() && tree.is_actionable(t.id))
        });

        Self {
            task_detail_modal_store: app_store.task_detail_modal,
            pending_view,
            reviewed_view,
            show_reviewed,
            actionable_count,
            hide_actionable,
            app_store,
        }
    }

    pub fn open_detail(&self, task_id: i64) {
        // Use pending view's tree for navigation IDs
        let tree = self.pending_view.tree.get_untracked();
        let interval = self
            .app_store
            .settings
            .review_interval_days()
            .get_untracked();
        let cutoff = Utc::now().date_naive() - chrono::Duration::days(interval);
        let projects = self.app_store.projects.get();

        let root_ids: Vec<i64> = tree
            .children_of(None)
            .all_ids()
            .copied()
            .filter(|id| {
                tree.get(*id)
                    .map(|t| {
                        !t.someday
                            && t.completed_at.is_none()
                            && match t.reviewed_at {
                                None => true,
                                Some(date) => date <= cutoff,
                            }
                            && if let Some(pid) = t.project_id {
                                projects
                                    .iter()
                                    .find(|p| p.id == pid)
                                    .is_some_and(|p| p.status == ProjectStatus::Active)
                            } else {
                                true
                            }
                    })
                    .unwrap_or(false)
            })
            .collect();
        self.task_detail_modal_store.open(task_id, root_ids);
    }

    pub fn toolbar_config(&self) -> ToolbarConfig {
        let app_store = self.app_store;
        ToolbarConfig {
            enabled: true,
            show_add_task: false,
            completed: None,
            actionable: Some(ActionableToggle {
                is_active: self.hide_actionable,
                count: self.actionable_count,
                on_toggle: Callback::new(move |()| {
                    app_store
                        .browser_storage
                        .toggle_bool(HIDE_NON_ACTIONABLE_KEY);
                }),
            }),
        }
    }
}
