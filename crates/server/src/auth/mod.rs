pub mod jwt;
pub mod middleware;

use north_domain::UserRole;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: i64,
    pub role: UserRole,
}
