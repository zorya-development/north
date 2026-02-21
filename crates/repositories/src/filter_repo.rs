use leptos::prelude::ServerFnError;
use north_dto::{DslSuggestion, SavedFilter};

use crate::{notify_on_error, TaskModel};

pub struct FilterRepository;

impl FilterRepository {
    pub async fn list() -> Result<Vec<SavedFilter>, ServerFnError> {
        notify_on_error(north_server_fns::filters::list_saved_filters().await)
    }

    pub async fn get(id: i64) -> Result<SavedFilter, ServerFnError> {
        notify_on_error(north_server_fns::filters::get_saved_filter(id).await)
    }

    pub async fn create(title: String, query: String) -> Result<SavedFilter, ServerFnError> {
        notify_on_error(north_server_fns::filters::create_saved_filter(title, query).await)
    }

    pub async fn update(
        id: i64,
        title: Option<String>,
        query: Option<String>,
    ) -> Result<SavedFilter, ServerFnError> {
        notify_on_error(north_server_fns::filters::update_saved_filter(id, title, query).await)
    }

    pub async fn delete(id: i64) -> Result<(), ServerFnError> {
        notify_on_error(north_server_fns::filters::delete_saved_filter(id).await)
    }

    pub async fn execute(query: String) -> Result<Vec<TaskModel>, ServerFnError> {
        notify_on_error(
            north_server_fns::filters::execute_filter(query)
                .await
                .map(|tasks| tasks.into_iter().map(TaskModel::from).collect()),
        )
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
