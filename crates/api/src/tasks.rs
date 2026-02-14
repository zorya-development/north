use leptos::prelude::*;
use north_domain::{CreateTask, Task, TaskWithMeta, UpdateTask};

#[server(ApiListTasksFn, "/api")]
pub async fn list_tasks() -> Result<Vec<TaskWithMeta>, ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    let mut tasks = north_services::TaskService::get_all(&pool, user_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let completed = north_services::TaskService::get_completed(&pool, user_id, None, false)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    tasks.extend(completed);
    Ok(tasks)
}

#[server(ApiGetTaskFn, "/api")]
pub async fn get_task(id: i64) -> Result<TaskWithMeta, ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_services::TaskService::get_by_id_with_meta(&pool, user_id, id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiCreateTaskFn, "/api")]
pub async fn create_task(input: CreateTask) -> Result<Task, ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_services::TaskService::create_task_full(&pool, user_id, &input)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiUpdateTaskFn, "/api")]
pub async fn update_task(id: i64, input: UpdateTask) -> Result<Task, ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_services::TaskService::update_task_full(&pool, user_id, id, &input)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiDeleteTaskFn, "/api")]
pub async fn delete_task(id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_services::TaskService::delete_task(&pool, user_id, id)
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
