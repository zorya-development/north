use leptos::prelude::ServerFnError;
use north_domain::{
    CreateProject, Project, ProjectStatus, ProjectWithColumns, UpdateProject,
};

pub struct ProjectRepository;

impl ProjectRepository {
    pub async fn list(
        status: Option<ProjectStatus>,
    ) -> Result<Vec<ProjectWithColumns>, ServerFnError> {
        north_api::projects::list_projects(status).await
    }

    pub async fn get(id: i64) -> Result<ProjectWithColumns, ServerFnError> {
        north_api::projects::get_project(id).await
    }

    pub async fn create(
        input: CreateProject,
    ) -> Result<ProjectWithColumns, ServerFnError> {
        north_api::projects::create_project(input).await
    }

    pub async fn update(
        id: i64,
        input: UpdateProject,
    ) -> Result<Project, ServerFnError> {
        north_api::projects::update_project(id, input).await
    }

    pub async fn delete(id: i64) -> Result<(), ServerFnError> {
        north_api::projects::delete_project(id).await
    }
}
