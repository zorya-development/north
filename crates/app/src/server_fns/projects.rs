use leptos::prelude::*;
use north_domain::{Project, Task};
#[cfg(feature = "ssr")]
use north_domain::{ProjectFilter, ProjectStatus, UpdateProject};

#[server(GetProjectsFn, "/api")]
pub async fn get_projects() -> Result<Vec<Project>, ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    let filter = ProjectFilter {
        status: Some(ProjectStatus::Active),
    };
    north_services::ProjectService::list(&pool, user_id, &filter)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(CreateProjectFn, "/api")]
pub async fn create_project(title: String) -> Result<Project, ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    let input = north_domain::CreateProject {
        title,
        description: None,
        view_type: None,
    };
    north_services::ProjectService::create(&pool, user_id, &input)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(SetTaskProjectFn, "/api")]
pub async fn set_task_project(task_id: i64, project_id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    north_services::TaskService::set_project(&pool, user_id, task_id, project_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ClearTaskProjectFn, "/api")]
pub async fn clear_task_project(task_id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    north_services::TaskService::clear_project(&pool, user_id, task_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ArchiveProjectFn, "/api")]
pub async fn archive_project(project_id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    let input = UpdateProject {
        status: Some(ProjectStatus::Archived),
        ..Default::default()
    };
    north_services::ProjectService::update(&pool, user_id, project_id, &input)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

#[server(UnarchiveProjectFn, "/api")]
pub async fn unarchive_project(project_id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    let input = UpdateProject {
        status: Some(ProjectStatus::Active),
        ..Default::default()
    };
    north_services::ProjectService::update(&pool, user_id, project_id, &input)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

#[server(GetProjectFn, "/api")]
pub async fn get_project(project_id: i64) -> Result<Project, ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    north_services::ProjectService::get_by_id(&pool, user_id, project_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(GetProjectTasksFn, "/api")]
pub async fn get_project_tasks(project_id: i64) -> Result<Vec<Task>, ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    north_services::TaskService::get_for_project(&pool, user_id, project_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(UpdateProjectDetailsFn, "/api")]
pub async fn update_project_details(
    project_id: i64,
    title: String,
    color: String,
) -> Result<(), ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    north_services::ProjectService::update_details(&pool, user_id, project_id, &title, &color)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(DeleteProjectFn, "/api")]
pub async fn delete_project(project_id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    north_services::ProjectService::delete(&pool, user_id, project_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(GetArchivedProjectsFn, "/api")]
pub async fn get_archived_projects() -> Result<Vec<Project>, ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    let filter = ProjectFilter {
        status: Some(ProjectStatus::Archived),
    };
    north_services::ProjectService::list(&pool, user_id, &filter)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}
