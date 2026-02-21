use leptos::prelude::ServerFnError;
use north_dto::Tag;

use crate::notify_on_error;

pub struct TagRepository;

impl TagRepository {
    pub async fn list() -> Result<Vec<Tag>, ServerFnError> {
        notify_on_error(north_server_fns::tags::list_tags().await)
    }
}
