use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[server(Login, "/api")]
pub async fn login(email: String, password: String) -> Result<(), ServerFnError> {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    use chrono::{Duration, Utc};
    use jsonwebtoken::{encode, EncodingKey, Header};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct Claims {
        sub: i64,
        role: String,
        exp: usize,
    }

    #[derive(sqlx::FromRow)]
    struct UserRow {
        id: i64,
        password_hash: String,
        role: String,
    }

    let pool = expect_context::<sqlx::PgPool>();

    let row = sqlx::query_as::<_, UserRow>(
        "SELECT id, password_hash, role::text as role \
         FROM users WHERE email = $1",
    )
    .bind(&email)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let row = row.ok_or_else(|| ServerFnError::new("Invalid credentials".to_string()))?;

    let parsed_hash =
        PasswordHash::new(&row.password_hash).map_err(|e| ServerFnError::new(e.to_string()))?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| ServerFnError::new("Invalid credentials".to_string()))?;

    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".to_string());

    let role_str = match row.role.as_str() {
        "admin" => "admin",
        _ => "user",
    };

    let exp = Utc::now() + Duration::days(7);
    let claims = Claims {
        sub: row.id,
        role: role_str.to_string(),
        exp: exp.timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let response_options = expect_context::<leptos_axum::ResponseOptions>();
    response_options.insert_header(
        http::header::SET_COOKIE,
        http::HeaderValue::from_str(&format!(
            "token={token}; HttpOnly; Path=/; Max-Age=604800; SameSite=Lax"
        ))
        .map_err(|e| ServerFnError::new(e.to_string()))?,
    );

    Ok(())
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let login_action = Action::new(|input: &(String, String)| {
        let (email, password) = input.clone();
        login(email, password)
    });

    let value = login_action.value();
    let pending = login_action.pending();
    let navigate = use_navigate();

    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());

    Effect::new(move || {
        if let Some(Ok(())) = value.get() {
            navigate("/inbox", Default::default());
        }
    });

    let error_message = move || {
        value.get().and_then(|result| match result {
            Err(e) => Some(e.to_string()),
            Ok(()) => None,
        })
    };

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        login_action.dispatch((email.get(), password.get()));
    };

    view! {
        <div class="min-h-screen bg-bg-primary flex items-center justify-center px-4">
            <div class="w-full max-w-sm bg-bg-secondary rounded-lg border border-border p-8">
                <h1 class="text-xl font-semibold text-text-primary text-center mb-6">
                    "North"
                </h1>

                {move || {
                    error_message().map(|msg| {
                        view! {
                            <div class="mb-4 p-3 bg-accent/10 border border-accent/30 \
                                        rounded-md text-accent text-sm">
                                {msg}
                            </div>
                        }
                    })
                }}

                <form on:submit=on_submit class="space-y-4">
                    <div>
                        <label
                            for="email"
                            class="block text-sm font-medium text-text-secondary mb-1"
                        >
                            "Email"
                        </label>
                        <input
                            id="email"
                            type="email"
                            required=true
                            prop:value=email
                            on:input=move |ev| {
                                set_email.set(event_target_value(&ev));
                            }
                            class="w-full px-3 py-2 text-sm bg-bg-input border border-border \
                                   rounded-md focus:border-accent focus:ring-1 \
                                   focus:ring-accent/20 outline-none transition-colors \
                                   text-text-primary placeholder:text-text-tertiary"
                            placeholder="you@example.com"
                        />
                    </div>

                    <div>
                        <label
                            for="password"
                            class="block text-sm font-medium text-text-secondary mb-1"
                        >
                            "Password"
                        </label>
                        <input
                            id="password"
                            type="password"
                            required=true
                            prop:value=password
                            on:input=move |ev| {
                                set_password.set(event_target_value(&ev));
                            }
                            class="w-full px-3 py-2 text-sm bg-bg-input border border-border \
                                   rounded-md focus:border-accent focus:ring-1 \
                                   focus:ring-accent/20 outline-none transition-colors \
                                   text-text-primary placeholder:text-text-tertiary"
                            placeholder="Enter your password"
                        />
                    </div>

                    <button
                        type="submit"
                        disabled=pending
                        class="w-full py-2 px-4 text-sm font-medium rounded-md \
                               bg-accent text-white hover:bg-accent-hover \
                               disabled:opacity-50 disabled:cursor-not-allowed \
                               transition-colors"
                    >
                        {move || {
                            if pending.get() { "Signing in..." } else { "Sign in" }
                        }}
                    </button>
                </form>
            </div>
        </div>
    }
}
