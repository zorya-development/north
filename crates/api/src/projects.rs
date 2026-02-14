use leptos::prelude::*;
use north_domain::{
    CreateProject, Project, ProjectStatus, ProjectWithColumns, UpdateProject,
};

#[server(ApiListProjectsFn, "/api")]
pub async fn list_projects(
    status: Option<ProjectStatus>,
) -> Result<Vec<ProjectWithColumns>, ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    let projects = match status {
        Some(ProjectStatus::Active) => {
            north_services::ProjectService::list_with_columns(&pool, user_id)
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))?
        }
        Some(ProjectStatus::Archived) => {
            let archived = north_services::ProjectService::get_archived(&pool, user_id)
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))?;
            archived
                .into_iter()
                .map(|p| ProjectWithColumns {
                    project: p,
                    columns: vec![],
                })
                .collect()
        }
        None => {
            let mut all =
                north_services::ProjectService::list_with_columns(&pool, user_id)
                    .await
                    .map_err(|e| ServerFnError::new(e.to_string()))?;
            let archived = north_services::ProjectService::get_archived(&pool, user_id)
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))?;
            all.extend(archived.into_iter().map(|p| ProjectWithColumns {
                project: p,
                columns: vec![],
            }));
            all
        }
    };
    Ok(projects)
}

#[server(ApiGetProjectFn, "/api")]
pub async fn get_project(id: i64) -> Result<ProjectWithColumns, ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_services::ProjectService::get_with_columns(&pool, user_id, id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiCreateProjectFn, "/api")]
pub async fn create_project(
    input: CreateProject,
) -> Result<ProjectWithColumns, ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_services::ProjectService::create(&pool, user_id, &input)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiUpdateProjectFn, "/api")]
pub async fn update_project(
    id: i64,
    input: UpdateProject,
) -> Result<Project, ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_services::ProjectService::update(&pool, user_id, id, &input)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiDeleteProjectFn, "/api")]
pub async fn delete_project(id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_services::ProjectService::delete(&pool, user_id, id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}
