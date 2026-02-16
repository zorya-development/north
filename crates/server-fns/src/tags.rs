use leptos::prelude::*;
use north_domain::Tag;

#[server(ApiListTagsFn, "/api")]
pub async fn list_tags() -> Result<Vec<Tag>, ServerFnError> {
    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_core::TagService::list(&pool, user_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}
