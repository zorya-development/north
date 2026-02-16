use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::CookieJar;
use north_dto::UserRole;
use serde_json::json;

use super::jwt::validate_token;
use super::AuthUser;
use crate::AppState;

pub async fn auth_middleware(
    State(state): State<AppState>,
    jar: CookieJar,
    mut request: Request,
    next: Next,
) -> Response {
    let token = jar.get("token").map(|c| c.value().to_string()).or_else(|| {
        request
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(|t| t.to_string())
    });

    let token = match token {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                axum::Json(json!({ "error": "Authentication required" })),
            )
                .into_response();
        }
    };

    let claims = match validate_token(&token, &state.jwt_secret) {
        Ok(c) => c,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                axum::Json(json!({ "error": "Invalid or expired token" })),
            )
                .into_response();
        }
    };

    let role = match claims.role.as_str() {
        "admin" => UserRole::Admin,
        _ => UserRole::User,
    };

    let auth_user = AuthUser {
        id: claims.sub,
        role,
    };

    request.extensions_mut().insert(auth_user);
    next.run(request).await
}
