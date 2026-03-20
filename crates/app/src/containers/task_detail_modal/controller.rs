use leptos::prelude::*;
use north_dto::RecurrenceType;
use north_stores::{AppStore, IdFilter, TaskModel, TaskStoreFilter};

use crate::containers::traversable_task_list::{CompletedToggle, ToolbarConfig};
use crate::libs::KeepCompletedVisible;

#[derive(Clone, Copy)]
pub struct TaskDetailModalController {
    app_store: AppStore,
    pub title_draft: RwSignal<String>,
    pub body_draft: RwSignal<String>,
    pub body_editing: RwSignal<bool>,
    pub focused_task_id: RwSignal<Option<i64>>,
    pub subtask_show_completed: RwSignal<bool>,
    pub subtask_filter: Signal<Callback<TaskModel, bool>>,
}

impl TaskDetailModalController {
    pub fn new(app_store: AppStore) -> Self {
        let subtask_show_completed = RwSignal::new(false);
        let keep_completed_signal = use_context::<KeepCompletedVisible>().map(|kc| kc.signal());
        let subtask_filter = Signal::derive(move || {
            let show = subtask_show_completed.get();
            let pinned = keep_completed_signal.map(|s| s.get()).unwrap_or_default();
            Callback::new(move |task: TaskModel| {
                task.completed_at.is_none() || show || pinned.contains(&task.id)
            })
        });

        Self {
            app_store,
            title_draft: RwSignal::new(String::new()),
            body_draft: RwSignal::new(String::new()),
            body_editing: RwSignal::new(false),
            focused_task_id: RwSignal::new(None),
            subtask_show_completed,
            subtask_filter,
        }
    }

    // --- Data access ---

    pub fn task(&self) -> Option<TaskModel> {
        let AppStore {
            task_detail_modal, ..
        } = self.app_store;
        task_detail_modal.task()
    }

    pub fn ancestors(&self) -> Vec<(i64, String, i64)> {
        let AppStore {
            task_detail_modal, ..
        } = self.app_store;
        task_detail_modal.ancestors()
    }

    pub fn has_stack(&self) -> bool {
        let AppStore {
            task_detail_modal, ..
        } = self.app_store;
        task_detail_modal.has_stack()
    }

    pub fn subtask_ids(&self, task_id: i64) -> Memo<Vec<i64>> {
        let all = self.all_subtasks(task_id);
        Memo::new(move |_| all.get().iter().map(|t| t.id).collect())
    }

    pub fn completed_subtask_count(&self, task_id: i64) -> Memo<usize> {
        let all = self.all_subtasks(task_id);
        Memo::new(move |_| {
            all.get()
                .iter()
                .filter(|t| t.completed_at.is_some())
                .count()
        })
    }

    pub fn default_project_signal(&self, project_id: Option<i64>) -> Signal<Option<i64>> {
        Signal::derive(move || project_id)
    }

    pub fn show_recurrence_modal(&self) -> bool {
        let AppStore { modal, .. } = self.app_store;
        modal.is_open("recurrence")
    }

    // --- Navigation ---

    pub fn close(&self) {
        let AppStore {
            task_detail_modal, ..
        } = self.app_store;
        task_detail_modal.close();
    }

    pub fn prev(&self) {
        let AppStore {
            task_detail_modal, ..
        } = self.app_store;
        task_detail_modal.prev();
    }

    pub fn next(&self) {
        let AppStore {
            task_detail_modal, ..
        } = self.app_store;
        task_detail_modal.next();
    }

    pub fn navigate_to_ancestor(&self, id: i64) {
        let AppStore {
            task_detail_modal, ..
        } = self.app_store;
        task_detail_modal.navigate_to_ancestor(id);
    }

    pub fn navigate_to_subtask(&self, id: i64) {
        let AppStore {
            task_detail_modal, ..
        } = self.app_store;
        task_detail_modal.navigate_to_subtask(id);
    }

    // --- Mutations ---

    pub fn save(&self) {
        let AppStore {
            task_detail_modal, ..
        } = self.app_store;
        let Some(t) = self.title_draft.try_get_untracked() else {
            return;
        };
        let Some(b) = self.body_draft.try_get_untracked() else {
            return;
        };
        let b = if b.trim().is_empty() { None } else { Some(b) };

        if let Some(task) = untrack(|| task_detail_modal.task()) {
            if task.title == t && task.body == b {
                return;
            }
        }

        task_detail_modal.update(t, b);
    }

    pub fn delete(&self) {
        let AppStore {
            task_detail_modal, ..
        } = self.app_store;
        task_detail_modal.delete();
    }

    pub fn set_project(&self, project_id: i64) {
        let AppStore {
            task_detail_modal, ..
        } = self.app_store;
        task_detail_modal.set_project(project_id);
    }

    pub fn clear_project(&self) {
        let AppStore {
            task_detail_modal, ..
        } = self.app_store;
        task_detail_modal.clear_project();
    }

    pub fn set_tags(&self, tags: Vec<String>) {
        let AppStore {
            task_detail_modal, ..
        } = self.app_store;
        task_detail_modal.set_tags(tags);
    }

    pub fn set_start_at(&self, start_at: String) {
        let AppStore {
            task_detail_modal, ..
        } = self.app_store;
        task_detail_modal.set_start_at(start_at);
    }

    pub fn clear_start_at(&self) {
        let AppStore {
            task_detail_modal, ..
        } = self.app_store;
        task_detail_modal.clear_start_at();
    }

    pub fn set_due_date(&self, val: String) {
        let AppStore {
            task_detail_modal, ..
        } = self.app_store;
        task_detail_modal.set_due_date(val);
    }

    pub fn clear_due_date(&self) {
        let AppStore {
            task_detail_modal, ..
        } = self.app_store;
        task_detail_modal.clear_due_date();
    }

    pub fn set_recurrence(&self, rt: Option<RecurrenceType>, rr: Option<String>) {
        let AppStore {
            task_detail_modal, ..
        } = self.app_store;
        task_detail_modal.set_recurrence(rt, rr);
    }

    pub fn set_sequential_limit(&self, n: i16) {
        let AppStore {
            task_detail_modal, ..
        } = self.app_store;
        task_detail_modal.set_sequential_limit(n);
    }

    pub fn open_recurrence_modal(&self) {
        let AppStore { modal, .. } = self.app_store;
        modal.open("recurrence");
    }

    pub fn close_recurrence_modal(&self) {
        let AppStore { modal, .. } = self.app_store;
        modal.close("recurrence");
    }

    pub fn reorder_task(&self, id: i64, key: String, parent: Option<Option<i64>>) {
        let AppStore { tasks, .. } = self.app_store;
        tasks.reorder_task(id, key, parent);
    }

    pub fn sync_drafts(&self, title: String, body: Option<String>) {
        let body = body.unwrap_or_default();
        if self.title_draft.try_get_untracked().as_ref() != Some(&title) {
            let _ = self.title_draft.try_set(title);
        }
        if self.body_draft.try_get_untracked().as_ref() != Some(&body) {
            let _ = self.body_draft.try_set(body);
        }
    }

    pub fn focus_if_new_task(&self, task_id: i64) -> bool {
        if self.focused_task_id.try_get_untracked() != Some(Some(task_id)) {
            let _ = self.focused_task_id.try_set(Some(task_id));
            return true;
        }
        false
    }

    pub fn subtask_toolbar_config(&self, completed_count: Memo<usize>) -> ToolbarConfig {
        let subtask_show_completed = self.subtask_show_completed;
        ToolbarConfig {
            enabled: true,
            show_add_task: true,
            completed: Some(CompletedToggle {
                is_active: subtask_show_completed.into(),
                count: completed_count,
                on_toggle: Callback::new(move |()| {
                    subtask_show_completed.update(|v| *v = !*v);
                }),
            }),
            actionable: None,
        }
    }

    // --- Private ---

    fn all_subtasks(&self, task_id: i64) -> Memo<Vec<TaskModel>> {
        let AppStore { tasks, .. } = self.app_store;
        tasks.filtered(TaskStoreFilter {
            parent_id: IdFilter::Is(task_id),
            ..Default::default()
        })
    }
}
