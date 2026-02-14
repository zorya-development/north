#[cfg(feature = "ssr")]
pub async fn get_auth_user_id() -> Result<i64, leptos::prelude::ServerFnError> {
    use axum_extra::extract::CookieJar;
    use jsonwebtoken::{decode, DecodingKey, Validation};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct Claims {
        sub: i64,
        role: String,
        exp: usize,
    }

    let jar: CookieJar = leptos_axum::extract().await?;

    let token = jar
        .get("token")
        .map(|c| c.value().to_string())
        .ok_or_else(|| {
            leptos::prelude::ServerFnError::new("Authentication required".to_string())
        })?;

    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".to_string());

    let token_data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| leptos::prelude::ServerFnError::new(format!("Invalid or expired token: {e}")))?;

    Ok(token_data.claims.sub)
}
