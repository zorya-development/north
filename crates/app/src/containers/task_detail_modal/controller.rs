use leptos::prelude::*;
use north_dto::RecurrenceType;
use north_stores::{
    AppStore, IdFilter, ModalStore, TaskDetailModalStore, TaskModel, TaskStoreFilter,
};

use crate::containers::traversable_task_list::ExtraVisibleIds;
use crate::libs::KeepCompletedVisible;

#[derive(Clone, Copy)]
pub struct TaskDetailModalController {
    store: TaskDetailModalStore,
    app_store: AppStore,
    modal: ModalStore,
    extra_visible_ids: RwSignal<Vec<i64>>,
    pub title_draft: RwSignal<String>,
    pub body_draft: RwSignal<String>,
    pub body_editing: RwSignal<bool>,
    pub focused_task_id: RwSignal<Option<i64>>,
    pub subtask_show_completed: RwSignal<bool>,
    pub subtask_filter: Signal<Callback<TaskModel, bool>>,
}

impl TaskDetailModalController {
    pub fn new(app_store: AppStore) -> Self {
        let extra_visible_ids = expect_context::<ExtraVisibleIds>().0;
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
            store: app_store.task_detail_modal,
            app_store,
            modal: app_store.modal,
            extra_visible_ids,
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
        self.store.task()
    }

    pub fn ancestors(&self) -> Vec<(i64, String, i64)> {
        self.store.ancestors()
    }

    pub fn has_stack(&self) -> bool {
        self.store.has_stack()
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

    pub fn total_subtask_count(&self, task_id: i64) -> Memo<usize> {
        let all = self.all_subtasks(task_id);
        Memo::new(move |_| all.get().len())
    }

    pub fn default_project_signal(&self, project_id: Option<i64>) -> Signal<Option<i64>> {
        Signal::derive(move || project_id)
    }

    pub fn show_recurrence_modal(&self) -> bool {
        self.modal.is_open("recurrence")
    }

    // --- Navigation ---

    pub fn close(&self) {
        self.store.close();
    }

    pub fn prev(&self) {
        self.store.prev();
    }

    pub fn next(&self) {
        self.store.next();
    }

    pub fn navigate_to_ancestor(&self, id: i64) {
        self.store.navigate_to_ancestor(id);
    }

    pub fn navigate_to_subtask(&self, id: i64) {
        self.store.navigate_to_subtask(id);
    }

    // --- Mutations ---

    pub fn save(&self) {
        let Some(t) = self.title_draft.try_get_untracked() else {
            return;
        };
        let Some(b) = self.body_draft.try_get_untracked() else {
            return;
        };
        let b = if b.trim().is_empty() { None } else { Some(b) };

        if let Some(task) = untrack(|| self.store.task()) {
            if task.title == t && task.body == b {
                return;
            }
        }

        self.store.update(t, b);
    }

    pub fn delete(&self) {
        self.store.delete();
    }

    pub fn set_project(&self, project_id: i64) {
        self.store.set_project(project_id);
    }

    pub fn clear_project(&self) {
        self.store.clear_project();
    }

    pub fn set_tags(&self, tags: Vec<String>) {
        self.store.set_tags(tags);
    }

    pub fn set_start_at(&self, start_at: String) {
        self.store.set_start_at(start_at);
    }

    pub fn clear_start_at(&self) {
        self.store.clear_start_at();
    }

    pub fn set_due_date(&self, val: String) {
        self.store.set_due_date(val);
    }

    pub fn clear_due_date(&self) {
        self.store.clear_due_date();
    }

    pub fn set_recurrence(&self, rt: Option<RecurrenceType>, rr: Option<String>) {
        self.store.set_recurrence(rt, rr);
    }

    pub fn set_sequential_limit(&self, n: i16) {
        self.store.set_sequential_limit(n);
    }

    pub fn open_recurrence_modal(&self) {
        self.modal.open("recurrence");
    }

    pub fn close_recurrence_modal(&self) {
        self.modal.close("recurrence");
    }

    pub fn reorder_task(&self, id: i64, key: String, parent: Option<Option<i64>>) {
        self.app_store.tasks.reorder_task(id, key, parent);
    }

    pub fn track_created_subtask(&self, id: i64) {
        self.extra_visible_ids.update(|ids| {
            if !ids.contains(&id) {
                ids.push(id);
            }
        });
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

    // --- Private ---

    fn all_subtasks(&self, task_id: i64) -> Memo<Vec<TaskModel>> {
        self.app_store.tasks.filtered(TaskStoreFilter {
            parent_id: IdFilter::Is(task_id),
            ..Default::default()
        })
    }
}
