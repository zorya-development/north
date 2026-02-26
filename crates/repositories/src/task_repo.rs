use leptos::prelude::ServerFnError;
use north_dto::{CreateTask, UpdateTask};

use crate::{notify_on_error, TaskModel};

pub struct TaskRepository;

impl TaskRepository {
    pub async fn list() -> Result<Vec<TaskModel>, ServerFnError> {
        notify_on_error(
            north_server_fns::tasks::list_tasks()
                .await
                .map(|tasks| tasks.into_iter().map(TaskModel::from).collect()),
        )
    }

    pub async fn get(id: i64) -> Result<TaskModel, ServerFnError> {
        notify_on_error(
            north_server_fns::tasks::get_task(id)
                .await
                .map(TaskModel::from),
        )
    }

    pub async fn create(input: CreateTask) -> Result<TaskModel, ServerFnError> {
        notify_on_error(
            north_server_fns::tasks::create_task(input)
                .await
                .map(TaskModel::from),
        )
    }

    pub async fn update(id: i64, input: UpdateTask) -> Result<TaskModel, ServerFnError> {
        notify_on_error(
            north_server_fns::tasks::update_task(id, input)
                .await
                .map(TaskModel::from),
        )
    }

    pub async fn complete(id: i64) -> Result<(), ServerFnError> {
        notify_on_error(north_server_fns::tasks::complete_task(id).await)
    }

    pub async fn uncomplete(id: i64) -> Result<(), ServerFnError> {
        notify_on_error(north_server_fns::tasks::uncomplete_task(id).await)
    }

    pub async fn delete(id: i64) -> Result<(), ServerFnError> {
        notify_on_error(north_server_fns::tasks::delete_task(id).await)
    }

    pub async fn set_tags(task_id: i64, tag_names: Vec<String>) -> Result<(), ServerFnError> {
        notify_on_error(north_server_fns::tasks::set_task_tags(task_id, tag_names).await)
    }
}
