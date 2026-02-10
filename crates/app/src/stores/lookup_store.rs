use leptos::prelude::*;
use north_domain::{Project, Tag};

use crate::server_fns::projects::get_projects;
use crate::server_fns::tags::get_tags;

#[derive(Clone)]
pub struct LookupStore {
    pub tags: Resource<Result<Vec<Tag>, ServerFnError>>,
    pub projects: Resource<Result<Vec<Project>, ServerFnError>>,
}

impl LookupStore {
    pub fn new() -> Self {
        let tags = Resource::new(|| (), |_| get_tags());
        let projects = Resource::new(|| (), |_| get_projects());
        Self { tags, projects }
    }

    pub fn refetch_tags(&self) {
        self.tags.refetch();
    }

    pub fn refetch_projects(&self) {
        self.projects.refetch();
    }
}
