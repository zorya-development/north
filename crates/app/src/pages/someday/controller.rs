use leptos::prelude::*;
use north_stores::{AppStore, TaskDetailModalStore, TaskModel};

use crate::containers::traversable_task_list::{ActionableToggle, ToolbarConfig};
use crate::controllers::TaskTreeView;
use crate::libs::{KeepCompletedVisible, KeepTaskVisible};

const HIDE_NON_ACTIONABLE_KEY: &str = "north:hide-non-actionable:someday";

#[derive(Clone, Copy)]
pub struct SomedayController {
    task_detail_modal_store: TaskDetailModalStore,
    pub view: TaskTreeView,
    pub actionable_count: Memo<usize>,
    pub hide_actionable: Signal<bool>,
    app_store: AppStore,
}

impl SomedayController {
    pub fn new(app_store: AppStore) -> Self {
        Effect::new(move |_| {
            app_store.tasks.refetch();
        });

        let hide_actionable =
            Signal::derive(move || app_store.browser_storage.get_bool(HIDE_NON_ACTIONABLE_KEY));
        let tree = app_store.tasks.task_tree;

        let keep_completed = KeepCompletedVisible::new();
        provide_context(keep_completed);
        let keep_completed_signal = keep_completed.signal();

        let filter = Signal::derive(move || {
            let hide = hide_actionable.get();
            let tree = tree.get();
            let pinned = keep_completed_signal.get();
            Callback::new(move |task: TaskModel| {
                if task.completed_at.is_some() {
                    return pinned.contains(&task.id);
                }
                // Root gating: only someday tasks
                if task.parent_id.is_none() && !task.someday {
                    return false;
                }
                if hide && !tree.is_actionable(task.id) {
                    return false;
                }
                true
            })
        });

        let view = TaskTreeView::new(app_store, filter);

        let actionable_count = Memo::new(move |_| {
            let tree = tree.get();
            tree.count_matching(|t| t.completed_at.is_none() && tree.is_actionable(t.id))
        });

        provide_context(KeepTaskVisible::new(view.extra_visible));

        Self {
            task_detail_modal_store: app_store.task_detail_modal,
            view,
            actionable_count,
            hide_actionable,
            app_store,
        }
    }

    pub fn open_detail(&self, task_id: i64) {
        let tree = self.view.tree.get_untracked();
        let root_ids: Vec<i64> = tree
            .children_of(None)
            .all_ids()
            .copied()
            .filter(|id| {
                tree.get(*id)
                    .map(|t| t.someday && t.completed_at.is_none())
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
