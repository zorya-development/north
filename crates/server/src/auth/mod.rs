pub mod jwt;
pub mod middleware;

use north_dto::UserRole;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: i64,
    #[allow(dead_code)]
    pub role: UserRole,
}
