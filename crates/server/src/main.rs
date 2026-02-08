mod auth;
mod error;
mod routes;
mod seed;

use axum::extract::FromRef;
use axum::Router;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tower_http::services::ServeDir;
use tracing_subscriber::EnvFilter;

/// Newtype wrapper so server functions can extract the JWT secret.
#[derive(Debug, Clone)]
pub struct JwtSecret(pub String);

/// Newtype wrapper so server functions can extract the upload directory.
#[derive(Debug, Clone)]
pub struct UploadDir(pub String);

/// Shared application state available to all Axum handlers.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
    pub upload_dir: String,
    pub leptos_options: LeptosOptions,
}

impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.leptos_options.clone()
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "dev-secret-change-me".to_string());
    let upload_dir = std::env::var("UPLOAD_DIR")
        .unwrap_or_else(|_| "./uploads".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Handle --seed flag
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--seed") {
        seed::seed_admin(&pool)
            .await
            .expect("Failed to seed admin");
        return;
    }

    // Leptos configuration
    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(north_app::App);

    let app_state = AppState {
        pool: pool.clone(),
        jwt_secret: jwt_secret.clone(),
        upload_dir: upload_dir.clone(),
        leptos_options: leptos_options.clone(),
    };

    // Context values for Leptos server functions
    let pool_ctx = pool.clone();
    let jwt_ctx = JwtSecret(jwt_secret);
    let upload_ctx = UploadDir(upload_dir);

    let app = Router::new()
        .nest("/api/auth", routes::public_api_router())
        .nest("/api", routes::protected_api_router(app_state.clone()))
        .leptos_routes_with_context(
            &app_state,
            routes,
            {
                let pool_ctx = pool_ctx.clone();
                let jwt_ctx = jwt_ctx.clone();
                let upload_ctx = upload_ctx.clone();
                move || {
                    provide_context(pool_ctx.clone());
                    provide_context(jwt_ctx.clone());
                    provide_context(upload_ctx.clone());
                }
            },
            {
                let leptos_options = leptos_options.clone();
                move || north_app::shell(leptos_options.clone())
            },
        )
        .nest_service(
            "/pkg",
            ServeDir::new(format!("{}/pkg", leptos_options.site_root)),
        )
        .nest_service("/public", ServeDir::new("public"))
        .with_state(app_state);

    let addr = leptos_options.site_addr;
    tracing::info!("Listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app.into_make_service())
        .await
        .expect("Server error");
}
