use std::path::Path;

use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
    sqlite::SqliteConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use crate::{
    db::{enums::UserStatus, models::User},
    ports::user::{IUserRepo, UserRepoError},
};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("src/db/migrations");

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

pub struct UserRepo {
    pool: DbPool,
}

impl UserRepo {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn get_connection(
        &self,
    ) -> Result<diesel::r2d2::PooledConnection<ConnectionManager<SqliteConnection>>, UserRepoError>
    {
        self.pool
            .get()
            .map_err(|e| UserRepoError::Database(e.to_string()))
    }
}

impl IUserRepo for UserRepo {
    fn select(&self, id: i64) -> Result<Option<User>, UserRepoError> {
        use crate::db::schema::user::dsl as user_dsl;
        let mut conn = self.get_connection()?;

        let result = user_dsl::user
            .filter(user_dsl::id.eq(&id))
            .first::<User>(&mut conn)
            .optional()
            .map_err(|e| UserRepoError::Database(e.to_string()))?;

        Ok(result)
    }

    fn insert(&self, new_user: &User) -> Result<(), UserRepoError> {
        use crate::db::schema::user::dsl as user_dsl;
        let mut conn = self.get_connection()?;

        diesel::insert_into(user_dsl::user)
            .values(new_user)
            .execute(&mut conn)
            .map_err(|e| UserRepoError::Database(e.to_string()))?;

        Ok(())
    }

    fn get_by_status(&self, status: UserStatus) -> Result<Vec<User>, UserRepoError> {
        use crate::db::schema::user::dsl as user_dsl;
        let mut conn = self.get_connection()?;

        let status_code: i32 = status.into();

        let results = user_dsl::user
            .filter(user_dsl::status.eq(&status_code))
            .load::<User>(&mut conn)
            .map_err(|e| UserRepoError::Database(e.to_string()))?;

        Ok(results)
    }

    fn set_status(&self, id: i64, status: UserStatus) -> Result<(), UserRepoError> {
        use crate::db::schema::user::dsl as user_dsl;
        let mut conn = self.get_connection()?;

        let status_code = match status {
            UserStatus::New => 0,
            UserStatus::Accepted => 1,
            UserStatus::Rejected => 2,
        };

        diesel::update(user_dsl::user.filter(user_dsl::id.eq(&id)))
            .set(user_dsl::status.eq(&status_code))
            .execute(&mut conn)
            .map_err(|e| UserRepoError::Database(e.to_string()))?;

        Ok(())
    }

    fn get(&self, id: i64) -> Result<User, UserRepoError> {
        use crate::db::schema::user::dsl as user_dsl;
        let mut conn = self.get_connection()?;

        user_dsl::user
            .find(&id)
            .first::<User>(&mut conn)
            .map_err(|e| UserRepoError::Database(e.to_string()))
    }
}

pub fn init_db(db_location: &str) -> Result<DbPool, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(parent) = Path::new(db_location).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let manager = ConnectionManager::<SqliteConnection>::new(db_location);
    let pool = Pool::builder()
        .max_size(1) // SQLite doesn't support multiple writers
        .build(manager)?;

    // Run migrations
    let mut conn = pool.get()?;
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| e.to_string())?;

    Ok(pool)
}
