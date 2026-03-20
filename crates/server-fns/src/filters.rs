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
    with_auth!(|pool, uid| {
        north_core::filter::autocomplete::get_dsl_suggestions(pool, uid, &query, cursor)
    })
}

#[server(ApiListSavedFiltersFn, "/api")]
pub async fn list_saved_filters() -> Result<Vec<SavedFilter>, ServerFnError> {
    with_auth!(|pool, uid| north_core::FilterService::list(pool, uid))
}

#[server(ApiGetSavedFilterFn, "/api")]
pub async fn get_saved_filter(id: i64) -> Result<SavedFilter, ServerFnError> {
    with_auth!(|pool, uid| north_core::FilterService::get_by_id(pool, uid, id))
}

#[server(ApiCreateSavedFilterFn, "/api")]
pub async fn create_saved_filter(
    title: String,
    query: String,
) -> Result<SavedFilter, ServerFnError> {
    with_auth!(|pool, uid| north_core::FilterService::create(pool, uid, &title, &query))
}

#[server(ApiUpdateSavedFilterFn, "/api")]
pub async fn update_saved_filter(
    id: i64,
    title: Option<String>,
    query: Option<String>,
) -> Result<SavedFilter, ServerFnError> {
    with_auth!(|pool, uid| {
        north_core::FilterService::update(pool, uid, id, title.as_deref(), query.as_deref(), None)
    })
}

#[server(ApiDeleteSavedFilterFn, "/api")]
pub async fn delete_saved_filter(id: i64) -> Result<(), ServerFnError> {
    with_auth!(|pool, uid| north_core::FilterService::delete(pool, uid, id))
}

#[server(ApiExecuteFilterFn, "/api")]
pub async fn execute_filter(query: String) -> Result<Vec<Task>, ServerFnError> {
    with_auth!(|pool, uid| { north_core::TaskService::execute_dsl_filter(pool, uid, &query) })
}
