use leptos::prelude::*;
use leptos::server_fn::codec::Json;
use north_domain::{CreateTask, Task, UpdateTask};

#[server(ApiListTasksFn, "/api")]
pub async fn list_tasks() -> Result<Vec<Task>, ServerFnError> {
    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    let filter = north_domain::TaskFilter::default();
    north_core::TaskService::list(&pool, user_id, &filter)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiGetTaskFn, "/api")]
pub async fn get_task(id: i64) -> Result<Task, ServerFnError> {
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

#[server(name = ApiUpdateTaskFn, prefix = "/api", input = Json)]
pub async fn update_task(id: i64, input: UpdateTask) -> Result<Task, ServerFnError> {
    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_core::TaskService::update(&pool, user_id, id, &input)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiCompleteTaskFn, "/api")]
pub async fn complete_task(id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_services::TaskService::complete_task(&pool, user_id, id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiUncompleteTaskFn, "/api")]
pub async fn uncomplete_task(id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_services::TaskService::uncomplete_task(&pool, user_id, id)
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
