pub mod models;
pub mod schema;
pub mod sql_types;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::AsyncPgConnection;

pub type DbPool = Pool<AsyncPgConnection>;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error(transparent)]
    Diesel(#[from] diesel::result::Error),

    #[error("Connection pool error: {0}")]
    Pool(#[from] diesel_async::pooled_connection::deadpool::PoolError),
}

pub type DbResult<T> = Result<T, DbError>;
