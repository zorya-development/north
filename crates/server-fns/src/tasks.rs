use leptos::prelude::*;
use leptos::server_fn::codec::Json;
use north_dto::{CreateTask, Task, UpdateTask};

#[cfg(feature = "ssr")]
use north_core::filter::text_parser::parse_tokens;

#[server(ApiListTasksFn, "/api")]
pub async fn list_tasks() -> Result<Vec<Task>, ServerFnError> {
    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    let filter = north_dto::TaskFilter::default();
    north_core::TaskService::list(&pool, user_id, &filter)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiGetTaskFn, "/api")]
pub async fn get_task(id: i64) -> Result<Task, ServerFnError> {
    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_core::TaskService::get_by_id(&pool, user_id, id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiCreateTaskFn, "/api")]
pub async fn create_task(input: CreateTask) -> Result<Task, ServerFnError> {
    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_core::TaskService::create(&pool, user_id, &input)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(name = ApiUpdateTaskFn, prefix = "/api", input = Json)]
pub async fn update_task(id: i64, input: UpdateTask) -> Result<Task, ServerFnError> {
    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_core::TaskService::update(&pool, user_id, id, &input)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiCompleteTaskFn, "/api")]
pub async fn complete_task(id: i64) -> Result<(), ServerFnError> {
    use chrono::Utc;
    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    let input = UpdateTask {
        completed_at: Some(Some(Utc::now())),
        ..Default::default()
    };
    north_core::TaskService::update(&pool, user_id, id, &input)
        .await
        .map(|_| ())
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiUncompleteTaskFn, "/api")]
pub async fn uncomplete_task(id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    let input = UpdateTask {
        completed_at: Some(None),
        ..Default::default()
    };
    north_core::TaskService::update(&pool, user_id, id, &input)
        .await
        .map(|_| ())
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiDeleteTaskFn, "/api")]
pub async fn delete_task(id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_core::TaskService::delete(&pool, user_id, id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiSetTaskTagsFn, "/api")]
pub async fn set_task_tags(task_id: i64, tag_names: Vec<String>) -> Result<(), ServerFnError> {
    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_core::TagService::sync_task_tags_pooled(&pool, user_id, task_id, &tag_names)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiReviewAllTasksFn, "/api")]
pub async fn review_all_tasks() -> Result<(), ServerFnError> {
    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_core::TaskService::review_all(&pool, user_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiCreateTaskWithTokensFn, "/api")]
pub async fn create_task_with_tokens(input: CreateTask) -> Result<Task, ServerFnError> {
    use chrono::Utc;
    use north_core::url_service;

    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;

    let parsed = parse_tokens(&input.title);
    let mut resolved_project_id = input.project_id;
    if let Some(ref project_name) = parsed.project {
        if let Ok(Some(pid)) =
            north_core::ProjectService::find_by_title(&pool, user_id, project_name).await
        {
            resolved_project_id = Some(pid);
        }
    }

    let title = if parsed.cleaned.is_empty() {
        input.title.clone()
    } else {
        parsed.cleaned
    };

    let has_urls = url_service::has_bare_urls(&title)
        || input
            .body
            .as_deref()
            .is_some_and(url_service::has_bare_urls);

    let create_input = CreateTask {
        title,
        body: input.body.clone(),
        project_id: resolved_project_id,
        ..input
    };

    let mut task = north_core::TaskService::create(&pool, user_id, &create_input)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if !parsed.tags.is_empty() {
        north_core::TagService::add_task_tags_pooled(&pool, user_id, task.id, &parsed.tags)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        task = north_core::TaskService::get_by_id(&pool, user_id, task.id)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
    }

    if has_urls {
        // Set the flag so the client knows to poll
        let flag_input = north_dto::UpdateTask {
            is_url_fetching: Some(Some(Utc::now())),
            ..Default::default()
        };
        task = north_core::TaskService::update(&pool, user_id, task.id, &flag_input)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        // Spawn background URL resolution
        let bg_pool = pool.clone();
        let task_id = task.id;
        let bg_title = task.title.clone();
        let bg_body = task.body.clone();
        tokio::spawn(async move {
            let resolved_title = url_service::resolve_urls_in_text(&bg_title).await;
            let resolved_body = match bg_body {
                Some(ref body) => Some(url_service::resolve_urls_in_text(body).await),
                None => None,
            };

            let update_input = north_dto::UpdateTask {
                title: Some(resolved_title),
                body: Some(resolved_body),
                is_url_fetching: Some(None),
                ..Default::default()
            };
            if let Err(e) =
                north_core::TaskService::update(&bg_pool, user_id, task_id, &update_input).await
            {
                tracing::error!(task_id, error = %e, "Background URL resolution failed");
            }
        });
    }

    Ok(task)
}

#[server(name = ApiUpdateTaskWithTokensFn, prefix = "/api", input = Json)]
pub async fn update_task_with_tokens(id: i64, input: UpdateTask) -> Result<Task, ServerFnError> {
    use chrono::Utc;
    use north_core::url_service;

    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;

    let mut resolved_input = input.clone();
    let mut tags_to_add = Vec::new();
    let mut has_urls = false;

    if let Some(ref title) = input.title {
        let parsed = parse_tokens(title);
        let cleaned = if parsed.cleaned.is_empty() {
            title.clone()
        } else {
            parsed.cleaned
        };
        if url_service::has_bare_urls(&cleaned) {
            has_urls = true;
        }
        resolved_input.title = Some(cleaned);
        tags_to_add = parsed.tags;

        if let Some(ref project_name) = parsed.project {
            if let Ok(Some(pid)) =
                north_core::ProjectService::find_by_title(&pool, user_id, project_name).await
            {
                resolved_input.project_id = Some(Some(pid));
            }
        }
    }

    if let Some(Some(ref body)) = input.body {
        if url_service::has_bare_urls(body) {
            has_urls = true;
        }
    }

    if has_urls {
        resolved_input.is_url_fetching = Some(Some(Utc::now()));
    }

    let mut task = north_core::TaskService::update(&pool, user_id, id, &resolved_input)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if !tags_to_add.is_empty() {
        north_core::TagService::add_task_tags_pooled(&pool, user_id, task.id, &tags_to_add)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        task = north_core::TaskService::get_by_id(&pool, user_id, task.id)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
    }

    if has_urls {
        let bg_pool = pool.clone();
        let task_id = task.id;
        let bg_title = task.title.clone();
        let bg_body = task.body.clone();
        tokio::spawn(async move {
            let resolved_title = url_service::resolve_urls_in_text(&bg_title).await;
            let resolved_body = match bg_body {
                Some(ref body) => Some(url_service::resolve_urls_in_text(body).await),
                None => None,
            };

            let update_input = north_dto::UpdateTask {
                title: Some(resolved_title),
                body: Some(resolved_body),
                is_url_fetching: Some(None),
                ..Default::default()
            };
            if let Err(e) =
                north_core::TaskService::update(&bg_pool, user_id, task_id, &update_input).await
            {
                tracing::error!(task_id, error = %e, "Background URL resolution failed");
            }
        });
    }

    Ok(task)
}

#[server(ApiAddTaskTagsFn, "/api")]
pub async fn add_task_tags(task_id: i64, tag_names: Vec<String>) -> Result<(), ServerFnError> {
    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_core::TagService::add_task_tags_pooled(&pool, user_id, task_id, &tag_names)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}
