use leptos::prelude::*;
use north_domain::TaskWithMeta;

use crate::AppStore;

#[derive(Clone, Copy)]
pub struct TaskDetailModalStore {
    app_store: AppStore,
    open_task_id: RwSignal<Option<i64>>,
    task_stack: RwSignal<Vec<i64>>,
    task_ids: RwSignal<Vec<i64>>,
    task_memo: RwSignal<Option<Memo<Option<TaskWithMeta>>>>,
}

impl TaskDetailModalStore {
    pub fn new(app_store: AppStore) -> Self {
        let open_task_id = RwSignal::new(None::<i64>);
        let task_stack = RwSignal::new(vec![]);
        let task_ids = RwSignal::new(vec![]);
        let task_memo: RwSignal<Option<Memo<Option<TaskWithMeta>>>> = RwSignal::new(None);

        let current_task_id = Self::current_task_id_memo(open_task_id, task_stack);

        // Update the task memo when current_task_id changes
        let task_store = app_store.tasks;
        Effect::new(move || match current_task_id.get() {
            Some(id) => task_memo.set(Some(task_store.get_by_id(id))),
            None => task_memo.set(None),
        });

        Self {
            app_store,
            open_task_id,
            task_stack,
            task_ids,
            task_memo,
        }
    }

    fn current_task_id_memo(
        open_task_id: RwSignal<Option<i64>>,
        task_stack: RwSignal<Vec<i64>>,
    ) -> Memo<Option<i64>> {
        Memo::new(move |_| {
            let stack = task_stack.get();
            if let Some(top) = stack.last() {
                Some(*top)
            } else {
                open_task_id.get()
            }
        })
    }

    pub fn current_task_id(&self) -> Option<i64> {
        let stack = self.task_stack.get();
        if let Some(top) = stack.last() {
            Some(*top)
        } else {
            self.open_task_id.get()
        }
    }

    pub fn task(&self) -> Option<TaskWithMeta> {
        self.task_memo.get().and_then(|memo| memo.get())
    }

    pub fn is_open(&self) -> bool {
        self.current_task_id().is_some()
    }

    pub fn has_stack(&self) -> bool {
        !self.task_stack.get().is_empty()
    }

    pub fn task_ids(&self) -> Vec<i64> {
        self.task_ids.get()
    }

    pub fn ancestors(&self) -> Vec<(i64, String, i64)> {
        match self.current_task_id() {
            Some(id) => self.app_store.tasks.get_ancestors(id),
            None => vec![],
        }
    }

    // ── Actions ──────────────────────────────────────────────────

    pub fn open(&self, task_id: i64, task_ids: Vec<i64>) {
        self.task_ids.set(task_ids);
        self.task_stack.set(vec![]);
        self.open_task_id.set(Some(task_id));
    }

    pub fn close(&self) {
        self.open_task_id.set(None);
        self.task_stack.set(vec![]);
    }

    pub fn prev(&self) {
        if !self.task_stack.get_untracked().is_empty() {
            return;
        }
        let ids = self.task_ids.get_untracked();
        if let Some(current) = self.open_task_id.get_untracked() {
            if let Some(idx) = ids.iter().position(|&id| id == current) {
                if idx > 0 {
                    self.open_task_id.set(Some(ids[idx - 1]));
                }
            }
        }
    }

    pub fn next(&self) {
        if !self.task_stack.get_untracked().is_empty() {
            return;
        }
        let ids = self.task_ids.get_untracked();
        if let Some(current) = self.open_task_id.get_untracked() {
            if let Some(idx) = ids.iter().position(|&id| id == current) {
                if idx + 1 < ids.len() {
                    self.open_task_id.set(Some(ids[idx + 1]));
                }
            }
        }
    }

    pub fn navigate_to_subtask(&self, subtask_id: i64) {
        let current = {
            let stack = self.task_stack.get_untracked();
            if let Some(top) = stack.last() {
                Some(*top)
            } else {
                self.open_task_id.get_untracked()
            }
        };
        if let Some(current) = current {
            self.task_stack.update(|s| s.push(current));
        }
        self.task_stack.update(|s| s.push(subtask_id));
    }

    pub fn navigate_to_ancestor(&self, ancestor_id: i64) {
        self.task_stack.update(|s| {
            if let Some(pos) = s.iter().position(|&id| id == ancestor_id) {
                s.truncate(pos + 1);
            } else {
                s.clear();
            }
        });
    }

    // ── Task mutations (delegate to app_store) ───────────────────

    pub fn toggle_complete(&self) {
        let Some(task) = self.task() else { return };
        let was_completed = task.task.completed_at.is_some();
        self.app_store
            .tasks
            .toggle_complete(task.task.id, was_completed);
    }

    pub fn delete(&self) {
        let Some(task) = self.task() else { return };
        self.app_store.tasks.delete_task(task.task.id);
        self.close();
    }

    pub fn update(&self, title: String, body: Option<String>) {
        let Some(task) = self.task() else { return };
        self.app_store.tasks.update_task(task.task.id, title, body);
    }

    pub fn set_start_at(&self, start_at: String) {
        let Some(task) = self.task() else { return };
        self.app_store.tasks.set_start_at(task.task.id, start_at);
    }

    pub fn clear_start_at(&self) {
        let Some(task) = self.task() else { return };
        self.app_store.tasks.clear_start_at(task.task.id);
    }

    pub fn set_project(&self, project_id: i64) {
        let Some(task) = self.task() else { return };
        self.app_store.tasks.set_project(task.task.id, project_id);
    }

    pub fn clear_project(&self) {
        let Some(task) = self.task() else { return };
        self.app_store.tasks.clear_project(task.task.id);
    }

    pub fn set_tags(&self, tag_names: Vec<String>) {
        let Some(task) = self.task() else { return };
        self.app_store.tasks.set_tags(task.task.id, tag_names);
    }

    pub fn set_due_date(&self, due_date: String) {
        let Some(task) = self.task() else { return };
        self.app_store.tasks.set_due_date(task.task.id, due_date);
    }

    pub fn clear_due_date(&self) {
        let Some(task) = self.task() else { return };
        self.app_store.tasks.clear_due_date(task.task.id);
    }

    pub fn set_sequential_limit(&self, limit: i16) {
        let Some(task) = self.task() else { return };
        self.app_store
            .tasks
            .set_sequential_limit(task.task.id, limit);
    }
}
