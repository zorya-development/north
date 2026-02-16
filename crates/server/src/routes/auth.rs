use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::extract::State;
use axum::Json;
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::CookieJar;
use north_core::UserService;
use north_domain::{AuthResponse, LoginRequest, User, UserRole, UserSettings};
use time::Duration;

use crate::error::AppError;
use crate::AppState;

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<LoginRequest>,
) -> Result<(CookieJar, Json<AuthResponse>), AppError> {
    let row = UserService::get_by_email(&state.pool, &body.email)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::Unauthorized("Invalid email or password".to_string()))?;

    let parsed_hash = PasswordHash::new(&row.password_hash)
        .map_err(|e| AppError::Internal(format!("Invalid password hash: {e}")))?;

    Argon2::default()
        .verify_password(body.password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Unauthorized("Invalid email or password".to_string()))?;

    let role: UserRole = row.role.into();
    let token = crate::auth::jwt::create_token(row.id, &role, &state.jwt_secret)?;

    let settings: UserSettings = serde_json::from_value(row.settings).unwrap_or_default();

    let user = User {
        id: row.id,
        email: row.email,
        name: row.name,
        role,
        settings,
        created_at: row.created_at,
        updated_at: row.updated_at,
    };

    let cookie = Cookie::build(("token", token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::days(7))
        .build();

    let jar = jar.add(cookie);
    Ok((jar, Json(AuthResponse { user })))
}

pub async fn logout(jar: CookieJar) -> CookieJar {
    let cookie = Cookie::build(("token", ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::seconds(0))
        .build();

    jar.add(cookie)
}
