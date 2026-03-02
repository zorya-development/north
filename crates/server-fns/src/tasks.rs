use leptos::prelude::*;
use leptos::server_fn::codec::Json;
use north_dto::{CreateTask, Task, UpdateTask};

#[server(ApiListTasksFn, "/api")]
pub async fn list_tasks() -> Result<Vec<Task>, ServerFnError> {
    let filter = north_dto::TaskFilter::default();
    with_auth!(|pool, uid| north_core::TaskService::list(pool, uid, &filter))
}

#[server(ApiGetTaskFn, "/api")]
pub async fn get_task(id: i64) -> Result<Task, ServerFnError> {
    with_auth!(|pool, uid| north_core::TaskService::get_by_id(pool, uid, id))
}

#[server(ApiCreateTaskFn, "/api")]
pub async fn create_task(input: CreateTask) -> Result<Task, ServerFnError> {
    with_auth!(|pool, uid| north_core::TaskService::create(pool, uid, &input))
}

#[server(name = ApiUpdateTaskFn, prefix = "/api", input = Json)]
pub async fn update_task(id: i64, input: UpdateTask) -> Result<Task, ServerFnError> {
    with_auth!(|pool, uid| north_core::TaskService::update(pool, uid, id, &input))
}

#[server(ApiCompleteTaskFn, "/api")]
pub async fn complete_task(id: i64) -> Result<(), ServerFnError> {
    use chrono::Utc;
    let input = UpdateTask {
        completed_at: Some(Some(Utc::now())),
        ..Default::default()
    };
    with_auth!(|pool, uid| async move {
        north_core::TaskService::update(pool, uid, id, &input)
            .await
            .map(|_| ())
    })
}

#[server(ApiUncompleteTaskFn, "/api")]
pub async fn uncomplete_task(id: i64) -> Result<(), ServerFnError> {
    let input = UpdateTask {
        completed_at: Some(None),
        ..Default::default()
    };
    with_auth!(|pool, uid| async move {
        north_core::TaskService::update(pool, uid, id, &input)
            .await
            .map(|_| ())
    })
}

#[server(ApiDeleteTaskFn, "/api")]
pub async fn delete_task(id: i64) -> Result<(), ServerFnError> {
    with_auth!(|pool, uid| north_core::TaskService::delete(pool, uid, id))
}

#[server(ApiSetTaskTagsFn, "/api")]
pub async fn set_task_tags(task_id: i64, tag_names: Vec<String>) -> Result<(), ServerFnError> {
    with_auth!(|pool, uid| {
        north_core::TagService::sync_task_tags_pooled(pool, uid, task_id, &tag_names)
    })
}

#[server(ApiAddTaskTagsFn, "/api")]
pub async fn add_task_tags(task_id: i64, tag_names: Vec<String>) -> Result<(), ServerFnError> {
    with_auth!(|pool, uid| {
        north_core::TagService::add_task_tags_pooled(pool, uid, task_id, &tag_names)
    })
}
