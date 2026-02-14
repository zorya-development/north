use chrono::Utc;
use leptos::prelude::*;
use leptos::task::spawn_local;
use north_domain::{CreateTask, TaskWithMeta, UpdateTask};
use north_repositories::TaskRepository;

#[derive(Clone, Copy)]
pub struct TaskStore {
    tasks: RwSignal<Vec<TaskWithMeta>>,
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

impl TaskStore {
    pub fn new() -> Self {
        Self {
            tasks: RwSignal::new(vec![]),
            loaded: RwSignal::new(false),
        }
    }

    // ── Reactive state methods ──────────────────────────────────

    pub fn load(&self, tasks: Vec<TaskWithMeta>) {
        self.tasks.set(tasks);
        self.loaded.set(true);
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded.get_untracked()
    }

    pub fn filtered(&self, filter: TaskStoreFilter) -> Memo<Vec<TaskWithMeta>> {
        let tasks = self.tasks;
        Memo::new(move |_| {
            tasks
                .get()
                .into_iter()
                .filter(|t| match &filter.project_id {
                    IdFilter::Any => true,
                    IdFilter::IsNull => t.task.project_id.is_none(),
                    IdFilter::Is(id) => t.task.project_id == Some(*id),
                })
                .filter(|t| match &filter.parent_id {
                    IdFilter::Any => true,
                    IdFilter::IsNull => t.task.parent_id.is_none(),
                    IdFilter::Is(id) => t.task.parent_id == Some(*id),
                })
                .filter(|t| match filter.is_completed {
                    None => true,
                    Some(true) => t.task.completed_at.is_some(),
                    Some(false) => t.task.completed_at.is_none(),
                })
                .collect()
        })
    }

    pub fn update_in_place(&self, id: i64, f: impl FnOnce(&mut TaskWithMeta)) {
        self.tasks.update(|tasks| {
            if let Some(t) = tasks.iter_mut().find(|t| t.task.id == id) {
                f(t);
            }
        });
    }

    pub fn remove(&self, id: i64) {
        self.tasks.update(|tasks| {
            tasks.retain(|t| t.task.id != id);
        });
    }

    pub fn add(&self, task: TaskWithMeta) {
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
            store.update_in_place(id, |t| t.task.completed_at = None);
            spawn_local(async move {
                let input = UpdateTask {
                    completed_at: Some(None),
                    ..Default::default()
                };
                if TaskRepository::update(id, input).await.is_ok() {
                    store.refetch_async().await;
                }
            });
        } else {
            let now = Utc::now();
            store.update_in_place(id, |t| {
                t.task.completed_at = Some(now);
            });
            spawn_local(async move {
                let input = UpdateTask {
                    completed_at: Some(Some(now)),
                    ..Default::default()
                };
                if TaskRepository::update(id, input).await.is_ok() {
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
            if TaskRepository::create(input).await.is_ok() {
                store.refetch_async().await;
            }
        });
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
            t.task.reviewed_at = Some(today);
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
