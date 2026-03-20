use leptos::prelude::*;
use north_stores::{AppStore, TaskDetailModalStore, TaskModel};

use crate::containers::traversable_task_list::{ActionableToggle, CompletedToggle, ToolbarConfig};
use crate::libs::TaskTreeView;
use crate::libs::{KeepCompletedVisible, KeepTaskVisible};

const HIDE_NON_ACTIONABLE_KEY: &str = "north:hide-non-actionable:all_tasks";

#[derive(Clone, Copy)]
pub struct AllTasksController {
    task_detail_modal_store: TaskDetailModalStore,
    pub view: TaskTreeView,
    pub show_completed: RwSignal<bool>,
    pub completed_count: Memo<usize>,
    pub actionable_count: Memo<usize>,
    pub hide_actionable: Signal<bool>,
    app_store: AppStore,
}

impl AllTasksController {
    pub fn new(app_store: AppStore) -> Self {
        Effect::new(move |_| {
            app_store.tasks.refetch();
        });

        let show_completed = RwSignal::new(false);
        let hide_actionable =
            Signal::derive(move || app_store.browser_storage.get_bool(HIDE_NON_ACTIONABLE_KEY));
        let tree = app_store.tasks.task_tree;

        let keep_completed = KeepCompletedVisible::new();
        provide_context(keep_completed);
        let keep_completed_signal = keep_completed.signal();

        let filter = Signal::derive(move || {
            let show = show_completed.get();
            let hide = hide_actionable.get();
            let tree = tree.get();
            let pinned = keep_completed_signal.get();
            Callback::new(move |task: TaskModel| {
                if task.completed_at.is_some() {
                    return show || pinned.contains(&task.id);
                }
                if hide && !tree.is_actionable(task.id) {
                    return false;
                }
                true
            })
        });

        let view = TaskTreeView::new(app_store, filter);

        let completed_count = Memo::new(move |_| {
            tree.get()
                .count_matching(|t| t.parent_id.is_none() && t.completed_at.is_some())
        });
        let actionable_count = Memo::new(move |_| {
            let tree = tree.get();
            tree.count_matching(|t| t.completed_at.is_none() && tree.is_actionable(t.id))
        });

        provide_context(KeepTaskVisible::new(view.extra_visible));

        Self {
            task_detail_modal_store: app_store.task_detail_modal,
            view,
            show_completed,
            completed_count,
            actionable_count,
            hide_actionable,
            app_store,
        }
    }

    pub fn open_detail(&self, task_id: i64) {
        let tree = self.view.tree.get_untracked();
        let root_ids: Vec<i64> = tree.children_of(None).all_ids().copied().collect();
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
