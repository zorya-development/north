use leptos::prelude::*;
use north_domain::{CreateTask, Task, TaskFilter, TaskWithMeta, UpdateTask};

#[server(ApiListTasksFn, "/api")]
pub async fn list_tasks() -> Result<Vec<TaskWithMeta>, ServerFnError> {
    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    let filter = TaskFilter::default();
    north_core::TaskService::list(&pool, user_id, &filter)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiGetTaskFn, "/api")]
pub async fn get_task(id: i64) -> Result<TaskWithMeta, ServerFnError> {
    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_core::TaskService::get_by_id(&pool, user_id, id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiCreateTaskFn, "/api")]
pub async fn create_task(input: CreateTask) -> Result<Task, ServerFnError> {
    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_core::TaskService::create(&pool, user_id, &input)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiUpdateTaskFn, "/api")]
pub async fn update_task(id: i64, input: UpdateTask) -> Result<Task, ServerFnError> {
    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_core::TaskService::update(&pool, user_id, id, &input)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiDeleteTaskFn, "/api")]
pub async fn delete_task(id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_core::TaskService::delete(&pool, user_id, id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiSetTaskTagsFn, "/api")]
pub async fn set_task_tags(task_id: i64, tag_names: Vec<String>) -> Result<(), ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_services::TagService::sync_task_tags_pooled(&pool, user_id, task_id, &tag_names)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}
