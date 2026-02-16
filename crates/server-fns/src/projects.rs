use leptos::prelude::*;
use north_domain::{CreateProject, Project, ProjectFilter, UpdateProject};

#[server(ApiListProjectsFn, "/api")]
pub async fn list_projects(filter: ProjectFilter) -> Result<Vec<Project>, ServerFnError> {
    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_core::ProjectService::list(&pool, user_id, &filter)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiGetProjectFn, "/api")]
pub async fn get_project(id: i64) -> Result<Project, ServerFnError> {
    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_core::ProjectService::get_by_id(&pool, user_id, id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiCreateProjectFn, "/api")]
pub async fn create_project(input: CreateProject) -> Result<Project, ServerFnError> {
    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_core::ProjectService::create(&pool, user_id, &input)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiUpdateProjectFn, "/api")]
pub async fn update_project(id: i64, input: UpdateProject) -> Result<Project, ServerFnError> {
    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_core::ProjectService::update(&pool, user_id, id, &input)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiDeleteProjectFn, "/api")]
pub async fn delete_project(id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_core::ProjectService::delete(&pool, user_id, id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}
