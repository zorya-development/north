use leptos::prelude::*;
use north_dto::{DslSuggestion, SavedFilter, Task};

#[server(ApiValidateFilterQueryFn, "/api")]
pub async fn validate_filter_query(query: String) -> Result<(), ServerFnError> {
    north_core::filter::parse_filter(&query)
        .map(|_| ())
        .map_err(|errs| {
            ServerFnError::new(
                errs.into_iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("; "),
            )
        })
}

#[server(ApiGetDslCompletionsFn, "/api")]
pub async fn get_dsl_completions(
    query: String,
    cursor: usize,
) -> Result<Vec<DslSuggestion>, ServerFnError> {
    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_core::filter::autocomplete::get_dsl_suggestions(&pool, user_id, &query, cursor)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiListSavedFiltersFn, "/api")]
pub async fn list_saved_filters() -> Result<Vec<SavedFilter>, ServerFnError> {
    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_core::FilterService::list(&pool, user_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiGetSavedFilterFn, "/api")]
pub async fn get_saved_filter(id: i64) -> Result<SavedFilter, ServerFnError> {
    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_core::FilterService::get_by_id(&pool, user_id, id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiCreateSavedFilterFn, "/api")]
pub async fn create_saved_filter(
    title: String,
    query: String,
) -> Result<SavedFilter, ServerFnError> {
    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_core::FilterService::create(&pool, user_id, &title, &query)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiUpdateSavedFilterFn, "/api")]
pub async fn update_saved_filter(
    id: i64,
    title: Option<String>,
    query: Option<String>,
) -> Result<SavedFilter, ServerFnError> {
    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_core::FilterService::update(&pool, user_id, id, title.as_deref(), query.as_deref(), None)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiDeleteSavedFilterFn, "/api")]
pub async fn delete_saved_filter(id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_core::FilterService::delete(&pool, user_id, id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiExecuteFilterFn, "/api")]
pub async fn execute_filter(query: String) -> Result<Vec<Task>, ServerFnError> {
    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_core::TaskService::execute_dsl_filter(&pool, user_id, &query)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}
