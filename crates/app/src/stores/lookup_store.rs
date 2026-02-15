use leptos::prelude::*;
use north_domain::Tag;

use crate::server_fns::tags::get_tags;

#[derive(Clone)]
pub struct LookupStore {
    pub tags: Resource<Result<Vec<Tag>, ServerFnError>>,
}

impl Default for LookupStore {
    fn default() -> Self {
        Self::new()
    }
}

impl LookupStore {
    pub fn new() -> Self {
        let tags = Resource::new(|| (), |_| get_tags());
        Self { tags }
    }

    pub fn refetch_tags(&self) {
        self.tags.refetch();
    }
}
