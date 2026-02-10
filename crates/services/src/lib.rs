pub mod column_service;
pub mod filter_service;
pub mod filter_translator;
pub mod project_service;
pub mod stats_service;
pub mod tag_service;
pub mod task_service;
pub mod user_service;

pub use column_service::ColumnService;
pub use filter_service::FilterService;
pub use project_service::ProjectService;
pub use stats_service::StatsService;
pub use tag_service::TagService;
pub use task_service::TaskService;
pub use user_service::UserService;

// Re-export DbPool so consumers only need north-services
pub use north_db::DbPool;
// Re-export UserRow for login page (needs password_hash)
pub use north_db::models::UserRow;

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error(transparent)]
    Db(#[from] north_db::DbError),

    #[error(transparent)]
    Diesel(#[from] diesel::result::Error),

    #[error("Connection pool error: {0}")]
    Pool(#[from] diesel_async::pooled_connection::deadpool::PoolError),
}

pub type ServiceResult<T> = Result<T, ServiceError>;
