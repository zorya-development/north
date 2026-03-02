use leptos::prelude::*;

#[server(ApiCheckAuthFn, "/api")]
pub async fn check_auth() -> Result<(), ServerFnError> {
    match get_auth_user_id().await {
        Ok(_) => Ok(()),
        Err(e) => {
            leptos_axum::redirect("/login");
            Err(e)
        }
    }
}

#[cfg(feature = "ssr")]
macro_rules! with_auth {
    ($f:expr) => {{
        #[allow(clippy::redundant_closure)]
        let __f = $f;
        let pool = leptos::prelude::expect_context::<north_core::DbPool>();
        let uid = crate::auth::get_auth_user_id().await?;
        __f(&pool, uid)
            .await
            .map_err(|e| leptos::prelude::ServerFnError::new(e.to_string()))
    }};
}

#[cfg(feature = "ssr")]
pub async fn get_auth_user_id() -> Result<i64, ServerFnError> {
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
        .ok_or_else(|| ServerFnError::new("Authentication required".to_string()))?;

    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".to_string());

    let token_data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| ServerFnError::new(format!("Invalid or expired token: {e}")))?;

    Ok(token_data.claims.sub)
}
