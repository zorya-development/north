use leptos::prelude::*;
use north_dto::{CreateProject, Project, ProjectFilter, UpdateProject};

#[server(ApiListProjectsFn, "/api")]
pub async fn list_projects(filter: ProjectFilter) -> Result<Vec<Project>, ServerFnError> {
    with_auth!(|pool, uid| north_core::ProjectService::list(pool, uid, &filter))
}

#[server(ApiGetProjectFn, "/api")]
pub async fn get_project(id: i64) -> Result<Project, ServerFnError> {
    with_auth!(|pool, uid| north_core::ProjectService::get_by_id(pool, uid, id))
}

#[server(ApiCreateProjectFn, "/api")]
pub async fn create_project(input: CreateProject) -> Result<Project, ServerFnError> {
    with_auth!(|pool, uid| north_core::ProjectService::create(pool, uid, &input))
}

#[server(ApiUpdateProjectFn, "/api")]
pub async fn update_project(id: i64, input: UpdateProject) -> Result<Project, ServerFnError> {
    with_auth!(|pool, uid| north_core::ProjectService::update(pool, uid, id, &input))
}

#[server(ApiDeleteProjectFn, "/api")]
pub async fn delete_project(id: i64) -> Result<(), ServerFnError> {
    with_auth!(|pool, uid| north_core::ProjectService::delete(pool, uid, id))
}
