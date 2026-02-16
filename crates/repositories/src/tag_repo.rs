use leptos::prelude::ServerFnError;
use north_dto::Tag;

pub struct TagRepository;

impl TagRepository {
    pub async fn list() -> Result<Vec<Tag>, ServerFnError> {
        north_server_fns::tags::list_tags().await
    }
}
