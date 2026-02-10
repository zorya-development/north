use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use north_db::DbPool;
use north_domain::UserSettings;
use north_services::UserService;

pub async fn seed_admin(pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    let exists = UserService::admin_exists(pool).await?;

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

    UserService::create_admin(pool, "admin@north.local", &password_hash, "Admin", settings)
        .await?;

    tracing::info!("Seeded admin user: admin@north.local / admin");
    Ok(())
}
