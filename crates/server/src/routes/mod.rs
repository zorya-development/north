mod auth;
mod projects;
mod stats;
mod tasks;

use axum::routing::{get, patch, post};
use axum::{middleware, Router};

use crate::auth::middleware::auth_middleware;
use crate::AppState;

pub fn public_api_router() -> Router<AppState> {
    Router::new()
        .route("/login", post(auth::login))
        .route("/logout", post(auth::logout))
}

pub fn protected_api_router(state: AppState) -> Router<AppState> {
    Router::new()
        // Task routes
        .route("/tasks", get(tasks::list_tasks).post(tasks::create_task))
        .route(
            "/tasks/:id",
            get(tasks::get_task)
                .patch(tasks::update_task)
                .delete(tasks::delete_task),
        )
        .route("/tasks/:id/review", patch(tasks::review_task))
        // Project routes
        .route(
            "/projects",
            get(projects::list_projects).post(projects::create_project),
        )
        .route(
            "/projects/:id",
            get(projects::get_project).patch(projects::update_project),
        )
        // Stats routes
        .route("/stats", get(stats::get_stats))
        // Auth middleware layer
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}
