use leptos::prelude::ServerFnError;
use north_dto::{DslSuggestion, SavedFilter, Task};

pub struct FilterRepository;

impl FilterRepository {
    pub async fn list() -> Result<Vec<SavedFilter>, ServerFnError> {
        north_server_fns::filters::list_saved_filters().await
    }

    pub async fn get(id: i64) -> Result<SavedFilter, ServerFnError> {
        north_server_fns::filters::get_saved_filter(id).await
    }

    pub async fn create(title: String, query: String) -> Result<SavedFilter, ServerFnError> {
        north_server_fns::filters::create_saved_filter(title, query).await
    }

    pub async fn update(
        id: i64,
        title: Option<String>,
        query: Option<String>,
    ) -> Result<SavedFilter, ServerFnError> {
        north_server_fns::filters::update_saved_filter(id, title, query).await
    }

    pub async fn delete(id: i64) -> Result<(), ServerFnError> {
        north_server_fns::filters::delete_saved_filter(id).await
    }

    pub async fn execute(query: String) -> Result<Vec<Task>, ServerFnError> {
        north_server_fns::filters::execute_filter(query).await
    }

    pub async fn validate_query(query: String) -> Result<(), ServerFnError> {
        north_server_fns::filters::validate_filter_query(query).await
    }

    pub async fn get_completions(
        query: String,
        cursor: usize,
    ) -> Result<Vec<DslSuggestion>, ServerFnError> {
        north_server_fns::filters::get_dsl_completions(query, cursor).await
    }
}
