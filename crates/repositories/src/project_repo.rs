use leptos::prelude::ServerFnError;
use north_domain::{CreateProject, Project, ProjectFilter, UpdateProject};

pub struct ProjectRepository;

impl ProjectRepository {
    pub async fn list(
        filter: ProjectFilter,
    ) -> Result<Vec<Project>, ServerFnError> {
        north_server_fns::projects::list_projects(filter).await
    }

    pub async fn get(id: i64) -> Result<Project, ServerFnError> {
        north_server_fns::projects::get_project(id).await
    }

    pub async fn create(
        input: CreateProject,
    ) -> Result<Project, ServerFnError> {
        north_server_fns::projects::create_project(input).await
    }

    pub async fn update(
        id: i64,
        input: UpdateProject,
    ) -> Result<Project, ServerFnError> {
        north_server_fns::projects::update_project(id, input).await
    }

    pub async fn delete(id: i64) -> Result<(), ServerFnError> {
        north_server_fns::projects::delete_project(id).await
    }
}
