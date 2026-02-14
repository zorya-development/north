use leptos::prelude::ServerFnError;
use north_domain::{CreateTask, Task, TaskWithMeta, UpdateTask};

pub struct TaskRepository;

impl TaskRepository {
    pub async fn list() -> Result<Vec<TaskWithMeta>, ServerFnError> {
        north_api::tasks::list_tasks().await
    }

    pub async fn get(id: i64) -> Result<TaskWithMeta, ServerFnError> {
        north_api::tasks::get_task(id).await
    }

    pub async fn create(input: CreateTask) -> Result<Task, ServerFnError> {
        north_api::tasks::create_task(input).await
    }

    pub async fn update(id: i64, input: UpdateTask) -> Result<Task, ServerFnError> {
        north_api::tasks::update_task(id, input).await
    }

    pub async fn delete(id: i64) -> Result<(), ServerFnError> {
        north_api::tasks::delete_task(id).await
    }

    pub async fn set_tags(task_id: i64, tag_names: Vec<String>) -> Result<(), ServerFnError> {
        north_api::tasks::set_task_tags(task_id, tag_names).await
    }
}
