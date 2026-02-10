use leptos::prelude::*;
use north_domain::{Task, TaskWithMeta};

#[server(GetInboxTasksFn, "/api")]
pub async fn get_inbox_tasks() -> Result<Vec<TaskWithMeta>, ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    north_services::TaskService::get_inbox(&pool, user_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(GetTodayTasksFn, "/api")]
pub async fn get_today_tasks() -> Result<Vec<TaskWithMeta>, ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    north_services::TaskService::get_today(&pool, user_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(GetAllTasksFn, "/api")]
pub async fn get_all_tasks() -> Result<Vec<TaskWithMeta>, ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    north_services::TaskService::get_all(&pool, user_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(CreateTaskFn, "/api")]
pub async fn create_task(title: String, body: Option<String>) -> Result<Task, ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    north_services::TaskService::create_task(&pool, user_id, title, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(UpdateTaskFn, "/api")]
pub async fn update_task(
    id: i64,
    title: String,
    body: Option<String>,
) -> Result<(), ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    north_services::TaskService::update_task(&pool, user_id, id, title, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(CompleteTaskFn, "/api")]
pub async fn complete_task(id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    north_services::TaskService::complete_task(&pool, user_id, id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(UncompleteTaskFn, "/api")]
pub async fn uncomplete_task(id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    north_services::TaskService::uncomplete_task(&pool, user_id, id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(DeleteTaskFn, "/api")]
pub async fn delete_task(id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    north_services::TaskService::delete_task(&pool, user_id, id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(SetTaskStartAtFn, "/api")]
pub async fn set_task_start_at(id: i64, start_at: String) -> Result<(), ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;

    let dt = chrono::NaiveDateTime::parse_from_str(&start_at, "%Y-%m-%dT%H:%M")
        .or_else(|_| chrono::NaiveDateTime::parse_from_str(&start_at, "%Y-%m-%dT%H:%M:%S"))
        .map_err(|e| ServerFnError::new(format!("Invalid datetime: {e}")))?;

    let dt_utc = dt.and_utc();

    north_services::TaskService::set_start_at(&pool, user_id, id, dt_utc)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ClearTaskStartAtFn, "/api")]
pub async fn clear_task_start_at(id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    north_services::TaskService::clear_start_at(&pool, user_id, id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(GetReviewTasksFn, "/api")]
pub async fn get_review_tasks() -> Result<Vec<TaskWithMeta>, ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    north_services::TaskService::get_review_due(&pool, user_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(GetRecentlyReviewedTasksFn, "/api")]
pub async fn get_recently_reviewed_tasks() -> Result<Vec<TaskWithMeta>, ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    north_services::TaskService::get_recently_reviewed(&pool, user_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ReviewTaskFn, "/api")]
pub async fn review_task(id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    north_services::TaskService::mark_reviewed(&pool, user_id, id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ReviewAllTasksFn, "/api")]
pub async fn review_all_tasks() -> Result<(), ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    north_services::TaskService::mark_all_reviewed(&pool, user_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}
