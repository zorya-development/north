use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use north_domain::UserSettings;
use sqlx::PgPool;

pub async fn seed_admin(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM users WHERE role = 'admin')",
    )
    .fetch_one(pool)
    .await?;

    if exists {
        tracing::info!("Admin user already exists, skipping seed");
        return Ok(());
    }

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(b"admin", &salt)
        .map_err(|e| format!("Failed to hash password: {e}"))?
        .to_string();

    let settings = serde_json::to_value(UserSettings::default())?;

    sqlx::query(
        "INSERT INTO users (email, password_hash, name, role, settings) \
         VALUES ($1, $2, $3, 'admin', $4)",
    )
    .bind("admin@north.local")
    .bind(&password_hash)
    .bind("Admin")
    .bind(&settings)
    .execute(pool)
    .await?;

    tracing::info!("Seeded admin user: admin@north.local / admin");
    Ok(())
}
