use leptos::prelude::*;
use north_stores::TaskModel;

use super::actionable::{is_actionable, ActionableController};
use crate::libs::KeepCompletedVisible;

#[derive(Clone, Copy)]
pub struct NodeFilterController {
    pub node_filter: Signal<Callback<TaskModel, bool>>,
    pub keep_completed: KeepCompletedVisible,
}

impl NodeFilterController {
    /// Create a node filter that combines completed-visibility, actionable-filtering,
    /// and keep-completed-pinning into a single callback.
    ///
    /// `show_completed` — `None` means never show completed (e.g. someday page).
    pub fn new(
        actionable: &ActionableController,
        show_completed: Option<Signal<bool>>,
        all_tasks: Memo<Vec<TaskModel>>,
    ) -> Self {
        let keep_completed = KeepCompletedVisible::new();
        provide_context(keep_completed);

        let hide = actionable.hide;
        let keep_signal = keep_completed.signal();
        let node_filter = Signal::derive(move || {
            let hide = hide.get();
            let show = show_completed.map(|s| s.get()).unwrap_or(false);
            let pinned = keep_signal.get();
            Callback::new(move |task: TaskModel| {
                if task.completed_at.is_some() {
                    return show || pinned.contains(&task.id);
                }
                if !hide {
                    return true;
                }
                is_actionable(&task, &all_tasks.get_untracked())
            })
        });

        Self {
            node_filter,
            keep_completed,
        }
    }
}
