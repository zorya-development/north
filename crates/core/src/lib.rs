pub mod project_service;
pub mod task_service;

pub use project_service::ProjectService;
pub use task_service::TaskService;

// Re-export DbPool so consumers only need north-core
pub use north_db::DbPool;

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
