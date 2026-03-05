use std::{path::Path, time::Duration};

use diesel::{
    r2d2::{ConnectionManager, Pool},
    sqlite::SqliteConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("src/db/migrations");
pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

pub fn connect(db_location: &str) -> Result<DbPool, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(parent) = Path::new(db_location).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let manager = ConnectionManager::<SqliteConnection>::new(db_location);
    let pool = Pool::builder()
        .connection_timeout(Duration::from_secs(10))
        .build(manager)?;

    // Run migrations
    let mut conn = pool.get()?;
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| e.to_string())?;

    Ok(pool)
}
