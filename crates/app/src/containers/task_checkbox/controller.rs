use leptos::prelude::*;
use north_stores::AppStore;

use crate::libs::KeepCompletedVisible;

#[derive(Clone, Copy)]
pub struct TaskCheckboxController {
    pub is_completed: Memo<bool>,
    pub progress: Memo<Option<(i64, i64)>>,
    task_id: i64,
    app_store: AppStore,
    keep_completed: Option<KeepCompletedVisible>,
}

impl TaskCheckboxController {
    pub fn new(app_store: AppStore, task_id: i64) -> Self {
        let task = app_store.tasks.get_by_id(task_id);

        let is_completed = Memo::new(move |_| {
            task.get()
                .map(|t| t.completed_at.is_some())
                .unwrap_or(false)
        });

        let progress = Memo::new(move |_| {
            task.get().and_then(|t| {
                if t.subtask_count > 0 {
                    Some((t.completed_subtask_count, t.subtask_count))
                } else {
                    None
                }
            })
        });

        let keep_completed = use_context::<KeepCompletedVisible>();

        Self {
            is_completed,
            progress,
            task_id,
            app_store,
            keep_completed,
        }
    }

    pub fn toggle_complete(&self) {
        let was_completed = self.is_completed.get_untracked();
        if !was_completed {
            if let Some(kc) = self.keep_completed {
                kc.keep(self.task_id);
            }
        }
        self.app_store
            .tasks
            .toggle_complete(self.task_id, was_completed);
    }
}
