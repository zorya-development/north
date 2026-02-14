use leptos::prelude::*;
use north_domain::{Column, Tag};

use crate::server_fns::projects::get_all_columns;
use crate::server_fns::tags::get_tags;

#[derive(Clone)]
pub struct LookupStore {
    pub tags: Resource<Result<Vec<Tag>, ServerFnError>>,
    pub columns: Resource<Result<Vec<Column>, ServerFnError>>,
}

impl LookupStore {
    pub fn new() -> Self {
        let tags = Resource::new(|| (), |_| get_tags());
        let columns = Resource::new(|| (), |_| get_all_columns());
        Self { tags, columns }
    }

    pub fn refetch_tags(&self) {
        self.tags.refetch();
    }
}
