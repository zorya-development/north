use leptos::prelude::*;
use north_domain::{SavedFilter, TaskWithMeta};

#[server(GetSavedFiltersFn, "/api")]
pub async fn get_saved_filters() -> Result<Vec<SavedFilter>, ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    north_services::FilterService::get_all(&pool, user_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(GetSavedFilterFn, "/api")]
pub async fn get_saved_filter(id: i64) -> Result<SavedFilter, ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    north_services::FilterService::get_by_id(&pool, user_id, id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(CreateSavedFilterFn, "/api")]
pub async fn create_saved_filter(
    title: String,
    query: String,
) -> Result<SavedFilter, ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    north_services::FilterService::create(&pool, user_id, &title, &query)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(UpdateSavedFilterFn, "/api")]
pub async fn update_saved_filter(
    id: i64,
    title: Option<String>,
    query: Option<String>,
) -> Result<SavedFilter, ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    north_services::FilterService::update(
        &pool,
        user_id,
        id,
        title.as_deref(),
        query.as_deref(),
        None,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(DeleteSavedFilterFn, "/api")]
pub async fn delete_saved_filter(id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    north_services::FilterService::delete(&pool, user_id, id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ExecuteFilterFn, "/api")]
pub async fn execute_filter(query: String) -> Result<Vec<TaskWithMeta>, ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    north_services::TaskService::execute_dsl_filter(&pool, user_id, &query)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}
