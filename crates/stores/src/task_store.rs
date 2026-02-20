use chrono::Utc;
use leptos::prelude::*;
use leptos::task::spawn_local;
use north_dto::{CreateTask, Task, UpdateTask};
use north_recurrence::RecurrenceType;
use north_repositories::TaskRepository;

#[derive(Clone, Copy)]
pub struct TaskStore {
    tasks: RwSignal<Vec<Task>>,
    loaded: RwSignal<bool>,
}

#[derive(Clone, Default)]
pub struct TaskStoreFilter {
    pub project_id: IdFilter,
    pub parent_id: IdFilter,
    pub is_completed: Option<bool>,
}

#[derive(Clone, Default)]
pub enum IdFilter {
    #[default]
    Any,
    IsNull,
    Is(i64),
}

impl Default for TaskStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskStore {
    pub fn new() -> Self {
        Self {
            tasks: RwSignal::new(vec![]),
            loaded: RwSignal::new(false),
        }
    }

    // ── Reactive state methods ──────────────────────────────────

    pub fn load(&self, tasks: Vec<Task>) {
        self.tasks.set(tasks);
        self.loaded.set(true);
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded.get_untracked()
    }

    pub fn loaded_signal(&self) -> Signal<bool> {
        self.loaded.into()
    }

    pub fn get_by_id(&self, id: i64) -> Memo<Option<Task>> {
        let tasks = self.tasks;
        Memo::new(move |_| tasks.get().into_iter().find(|t| t.id == id))
    }

    /// Walk parent_id chain from the store. Returns `(id, title, subtask_count)` list
    /// from root down to the immediate parent (excludes the task itself).
    pub fn get_ancestors(&self, id: i64) -> Vec<(i64, String, i64)> {
        let all = self.tasks.get_untracked();
        let mut ancestors = Vec::new();
        let mut current_id = id;

        for _ in 0..10 {
            let Some(task) = all.iter().find(|t| t.id == current_id) else {
                break;
            };
            let Some(parent_id) = task.parent_id else {
                break;
            };
            let Some(parent) = all.iter().find(|t| t.id == parent_id) else {
                break;
            };
            ancestors.push((parent.id, parent.title.clone(), parent.subtask_count));
            current_id = parent_id;
        }

        ancestors.reverse();
        ancestors
    }

    pub fn filtered(&self, filter: TaskStoreFilter) -> Memo<Vec<Task>> {
        let tasks = self.tasks;
        Memo::new(move |_| {
            tasks
                .get()
                .into_iter()
                .filter(|t| match &filter.project_id {
                    IdFilter::Any => true,
                    IdFilter::IsNull => t.project_id.is_none(),
                    IdFilter::Is(id) => t.project_id == Some(*id),
                })
                .filter(|t| match &filter.parent_id {
                    IdFilter::Any => true,
                    IdFilter::IsNull => t.parent_id.is_none(),
                    IdFilter::Is(id) => t.parent_id == Some(*id),
                })
                .filter(|t| match filter.is_completed {
                    None => true,
                    Some(true) => t.completed_at.is_some(),
                    Some(false) => t.completed_at.is_none(),
                })
                .collect()
        })
    }

    pub fn update_in_place(&self, id: i64, f: impl FnOnce(&mut Task)) {
        self.tasks.update(|tasks| {
            if let Some(t) = tasks.iter_mut().find(|t| t.id == id) {
                f(t);
            }
        });
    }

    pub fn remove(&self, id: i64) {
        self.tasks.update(|tasks| {
            tasks.retain(|t| t.id != id);
        });
    }

    pub fn add(&self, task: Task) {
        self.tasks.update(|tasks| {
            tasks.push(task);
        });
    }

    // ── Domain service methods ──────────────────────────────────

    pub fn refetch(&self) {
        let store = *self;
        spawn_local(async move {
            if let Ok(tasks) = TaskRepository::list().await {
                store.load(tasks);
            }
        });
    }

    pub fn toggle_complete(&self, id: i64, was_completed: bool) {
        let store = *self;
        if was_completed {
            store.update_in_place(id, |t| t.completed_at = None);
            spawn_local(async move {
                if TaskRepository::uncomplete(id).await.is_ok() {
                    store.refetch_async().await;
                }
            });
        } else {
            let now = Utc::now();
            store.update_in_place(id, |t| {
                t.completed_at = Some(now);
            });
            spawn_local(async move {
                if TaskRepository::complete(id).await.is_ok() {
                    store.refetch_async().await;
                }
            });
        }
    }

    pub fn delete_task(&self, id: i64) {
        self.remove(id);
        spawn_local(async move {
            let _ = TaskRepository::delete(id).await;
        });
    }

    pub fn update_task(&self, id: i64, title: String, body: Option<String>) {
        let store = *self;
        spawn_local(async move {
            let input = UpdateTask {
                title: Some(title),
                body: Some(body),
                ..Default::default()
            };
            if TaskRepository::update(id, input).await.is_ok() {
                store.refetch_async().await;
            }
        });
    }

    pub fn create_task(&self, input: CreateTask) {
        let store = *self;
        spawn_local(async move {
            if let Ok(task) = TaskRepository::create(input).await {
                if let Some(pid) = task.parent_id {
                    store.update_in_place(pid, |t| t.subtask_count += 1);
                }
                store.add(task);
            }
        });
    }

    /// Create a task and return it. Does NOT update the parent's subtask_count
    /// to avoid triggering a re-render of the parent task item (which would
    /// destroy any inline input that is currently focused).
    pub async fn create_task_async(&self, input: CreateTask) -> Option<Task> {
        match TaskRepository::create(input).await {
            Ok(task) => {
                self.add(task.clone());
                Some(task)
            }
            Err(_) => None,
        }
    }

    pub fn set_start_at(&self, id: i64, start_at: String) {
        let store = *self;
        spawn_local(async move {
            let dt = chrono::NaiveDateTime::parse_from_str(&start_at, "%Y-%m-%dT%H:%M")
                .or_else(|_| chrono::NaiveDateTime::parse_from_str(&start_at, "%Y-%m-%dT%H:%M:%S"));
            if let Ok(dt) = dt {
                let input = UpdateTask {
                    start_at: Some(Some(dt.and_utc())),
                    ..Default::default()
                };
                if TaskRepository::update(id, input).await.is_ok() {
                    store.refetch_async().await;
                }
            }
        });
    }

    pub fn clear_start_at(&self, id: i64) {
        let store = *self;
        spawn_local(async move {
            let input = UpdateTask {
                start_at: Some(None),
                ..Default::default()
            };
            if TaskRepository::update(id, input).await.is_ok() {
                store.refetch_async().await;
            }
        });
    }

    pub fn set_project(&self, task_id: i64, project_id: i64) {
        let store = *self;
        spawn_local(async move {
            let input = UpdateTask {
                project_id: Some(Some(project_id)),
                ..Default::default()
            };
            if TaskRepository::update(task_id, input).await.is_ok() {
                store.refetch_async().await;
            }
        });
    }

    pub fn clear_project(&self, task_id: i64) {
        let store = *self;
        spawn_local(async move {
            let input = UpdateTask {
                project_id: Some(None),
                ..Default::default()
            };
            if TaskRepository::update(task_id, input).await.is_ok() {
                store.refetch_async().await;
            }
        });
    }

    pub fn set_tags(&self, task_id: i64, tag_names: Vec<String>) {
        let store = *self;
        spawn_local(async move {
            if TaskRepository::set_tags(task_id, tag_names).await.is_ok() {
                store.refetch_async().await;
            }
        });
    }

    pub fn review_task(&self, id: i64) {
        let store = *self;
        let today = Utc::now().date_naive();
        store.update_in_place(id, move |t| {
            t.reviewed_at = Some(today);
        });
        spawn_local(async move {
            let input = UpdateTask {
                reviewed_at: Some(Some(today)),
                ..Default::default()
            };
            if TaskRepository::update(id, input).await.is_ok() {
                store.refetch_async().await;
            }
        });
    }

    pub fn set_due_date(&self, id: i64, due_date: String) {
        let store = *self;
        spawn_local(async move {
            let date = chrono::NaiveDate::parse_from_str(&due_date, "%Y-%m-%d");
            if let Ok(date) = date {
                let input = UpdateTask {
                    due_date: Some(Some(date)),
                    ..Default::default()
                };
                if TaskRepository::update(id, input).await.is_ok() {
                    store.refetch_async().await;
                }
            }
        });
    }

    pub fn clear_due_date(&self, id: i64) {
        let store = *self;
        spawn_local(async move {
            let input = UpdateTask {
                due_date: Some(None),
                ..Default::default()
            };
            if TaskRepository::update(id, input).await.is_ok() {
                store.refetch_async().await;
            }
        });
    }

    pub fn set_sequential_limit(&self, id: i64, limit: i16) {
        let store = *self;
        spawn_local(async move {
            let input = UpdateTask {
                sequential_limit: Some(limit),
                ..Default::default()
            };
            if TaskRepository::update(id, input).await.is_ok() {
                store.refetch_async().await;
            }
        });
    }

    pub fn set_recurrence(
        &self,
        id: i64,
        recurrence_type: Option<RecurrenceType>,
        recurrence_rule: Option<String>,
    ) {
        let store = *self;
        spawn_local(async move {
            let input = UpdateTask {
                recurrence_type: Some(recurrence_type),
                recurrence_rule: Some(recurrence_rule),
                ..Default::default()
            };
            if TaskRepository::update(id, input).await.is_ok() {
                store.refetch_async().await;
            }
        });
    }

    pub fn reorder_task(&self, task_id: i64, sort_key: String, parent_id: Option<Option<i64>>) {
        let store = *self;
        spawn_local(async move {
            let input = UpdateTask {
                sort_key: Some(sort_key),
                parent_id,
                ..Default::default()
            };
            if TaskRepository::update(task_id, input).await.is_ok() {
                store.refetch_async().await;
            }
        });
    }

    // ── Internal helpers ────────────────────────────────────────

    async fn refetch_async(&self) {
        if let Ok(tasks) = TaskRepository::list().await {
            self.load(tasks);
        }
    }
}
