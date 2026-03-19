use leptos::prelude::*;
use north_dto::Project;
use north_stores::{AppStore, TaskDetailModalStore, TaskModel};

use crate::containers::traversable_task_list::{ActionableToggle, CompletedToggle, ToolbarConfig};
use crate::controllers::TaskTreeView;
use crate::libs::{KeepCompletedVisible, KeepTaskVisible};

const HIDE_NON_ACTIONABLE_KEY: &str = "north:hide-non-actionable:project";

#[derive(Clone, Copy)]
pub struct ProjectController {
    task_detail_modal_store: TaskDetailModalStore,
    pub project: Memo<Option<Project>>,
    pub view: TaskTreeView,
    pub show_completed: RwSignal<bool>,
    pub completed_count: Memo<usize>,
    pub actionable_count: Memo<usize>,
    pub hide_actionable: Signal<bool>,
    app_store: AppStore,
    project_id: Signal<i64>,
}

impl ProjectController {
    pub fn new(app_store: AppStore, project_id: Signal<i64>) -> Self {
        Effect::new(move |_| {
            app_store.tasks.refetch();
        });

        let project = Memo::new(move |_| {
            let pid = project_id.get();
            app_store.projects.get().into_iter().find(|p| p.id == pid)
        });

        let show_completed = RwSignal::new(false);
        let hide_actionable =
            Signal::derive(move || app_store.browser_storage.get_bool(HIDE_NON_ACTIONABLE_KEY));
        let tree = app_store.tasks.task_tree;

        let keep_completed = KeepCompletedVisible::new();
        provide_context(keep_completed);
        let keep_completed_signal = keep_completed.signal();

        let filter = Signal::derive(move || {
            let pid = project_id.get();
            let show = show_completed.get();
            let hide = hide_actionable.get();
            let tree = tree.get();
            let pinned = keep_completed_signal.get();
            Callback::new(move |task: TaskModel| {
                if task.completed_at.is_some() {
                    return show || pinned.contains(&task.id);
                }
                // Root gating: only tasks in this project
                if task.parent_id.is_none() && task.project_id != Some(pid) {
                    return false;
                }
                if hide && !tree.is_actionable(task.id) {
                    return false;
                }
                true
            })
        });

        let view = TaskTreeView::new(app_store, filter);
        let extra_visible = view.extra_visible;

        // Auto-detect tasks that leave the project (e.g. moved to another project)
        let project_root_ids = Memo::new(move |_| {
            let pid = project_id.get();
            let tree = tree.get();
            tree.children_of(None)
                .all_ids()
                .copied()
                .filter(|id| {
                    tree.get(*id)
                        .map(|t| t.project_id == Some(pid))
                        .unwrap_or(false)
                })
                .collect::<Vec<_>>()
        });

        let prev_filtered_ids: RwSignal<Vec<i64>> = RwSignal::new(vec![]);

        // Clear extra_visible when navigating to a different project
        Effect::new(move |prev_pid: Option<i64>| {
            let pid = project_id.get();
            if prev_pid.is_some_and(|p| p != pid) {
                extra_visible.set(vec![]);
                prev_filtered_ids.set(vec![]);
            }
            pid
        });

        Effect::new(move |_| {
            let current = project_root_ids.get();
            let prev = prev_filtered_ids.get_untracked();
            for &id in &prev {
                if !current.contains(&id) {
                    extra_visible.update(|ids| {
                        if !ids.contains(&id) {
                            ids.push(id);
                        }
                    });
                }
            }
            prev_filtered_ids.set(current);
        });

        provide_context(KeepTaskVisible::new(extra_visible));

        let completed_count = Memo::new(move |_| {
            let pid = project_id.get();
            tree.get().count_matching(|t| {
                t.parent_id.is_none() && t.project_id == Some(pid) && t.completed_at.is_some()
            })
        });
        let actionable_count = Memo::new(move |_| {
            let tree = tree.get();
            tree.count_matching(|t| t.completed_at.is_none() && tree.is_actionable(t.id))
        });

        Self {
            task_detail_modal_store: app_store.task_detail_modal,
            project,
            view,
            show_completed,
            completed_count,
            actionable_count,
            hide_actionable,
            app_store,
            project_id,
        }
    }

    pub fn open_detail(&self, task_id: i64) {
        let pid = self.project_id.get_untracked();
        let tree = self.view.tree.get_untracked();
        let root_ids: Vec<i64> = tree
            .children_of(None)
            .all_ids()
            .copied()
            .filter(|id| {
                tree.get(*id)
                    .map(|t| t.project_id == Some(pid))
                    .unwrap_or(false)
            })
            .collect();
        self.task_detail_modal_store.open(task_id, root_ids);
    }

    pub fn reorder_task(&self, task_id: i64, sort_key: String, parent_id: Option<Option<i64>>) {
        self.app_store
            .tasks
            .reorder_task(task_id, sort_key, parent_id);
    }

    pub fn toolbar_config(&self) -> ToolbarConfig {
        let show_completed = self.show_completed;
        let app_store = self.app_store;
        ToolbarConfig {
            enabled: true,
            show_add_task: true,
            completed: Some(CompletedToggle {
                is_active: show_completed.into(),
                count: self.completed_count,
                on_toggle: Callback::new(move |()| {
                    show_completed.update(|v| *v = !*v);
                }),
            }),
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
