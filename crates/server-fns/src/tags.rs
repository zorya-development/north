use leptos::prelude::*;
use north_dto::Tag;

#[server(ApiListTagsFn, "/api")]
pub async fn list_tags() -> Result<Vec<Tag>, ServerFnError> {
    with_auth!(|pool, uid| north_core::TagService::list(pool, uid))
}
