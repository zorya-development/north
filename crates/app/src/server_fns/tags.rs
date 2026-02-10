use leptos::prelude::*;
use north_domain::Tag;

#[server(GetTagsFn, "/api")]
pub async fn get_tags() -> Result<Vec<Tag>, ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    north_services::TagService::get_all(&pool, user_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(SetTaskTagsFn, "/api")]
pub async fn set_task_tags(
    task_id: i64,
    tag_names: Vec<String>,
) -> Result<(), ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    north_services::TagService::sync_task_tags_pooled(&pool, user_id, task_id, &tag_names)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}
