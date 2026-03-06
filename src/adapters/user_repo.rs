use diesel::{prelude::*, sqlite::SqliteConnection};

use crate::{
    db::{enums::UserStatus, models::User, schema::user, ConnPool},
    ports::{user::UserRepoTrait, RepoError},
};

type DbConn = diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<SqliteConnection>>;

pub struct UserRepo {
    pool: ConnPool,
}

impl UserRepo {
    pub fn new(pool: ConnPool) -> Self {
        Self { pool }
    }

    fn conn(&self) -> Result<DbConn, RepoError> {
        self.pool
            .get()
            .map_err(|e| RepoError::Database(e.to_string()))
    }
}

impl UserRepoTrait for UserRepo {
    fn get(&self, id: i64) -> Result<Option<User>, RepoError> {
        let mut conn = self.conn()?;

        let result = user::table
            .filter(user::id.eq(&id))
            .first::<User>(&mut conn)
            .optional()
            .map_err(|e| RepoError::Database(e.to_string()))?;

        Ok(result)
    }

    fn insert(&self, new_user: &User) -> Result<(), RepoError> {
        let mut conn = self.conn()?;

        diesel::insert_into(user::table)
            .values(new_user)
            .execute(&mut conn)
            .map_err(|e| RepoError::Database(e.to_string()))?;

        Ok(())
    }

    fn get_by_status(&self, status: UserStatus) -> Result<Vec<User>, RepoError> {
        let mut conn = self.conn()?;
        let status_code: i32 = status.into();

        let results = user::table
            .filter(user::status.eq(&status_code))
            .load::<User>(&mut conn)
            .map_err(|e| RepoError::Database(e.to_string()))?;

        Ok(results)
    }

    fn set_status(&self, id: i64, status: UserStatus) -> Result<(), RepoError> {
        let mut conn = self.conn()?;
        let status_code: i32 = status.into();

        diesel::update(user::table.filter(user::id.eq(&id)))
            .set(user::status.eq(&status_code))
            .execute(&mut conn)
            .map_err(|e| RepoError::Database(e.to_string()))?;

        Ok(())
    }

    fn get_status(&self, id: i64) -> Result<String, RepoError> {
        let mut conn = self.conn()?;

        let status_code = user::table
            .filter(user::id.eq(&id))
            .select(user::status)
            .first::<i32>(&mut conn)
            .map_err(|e| RepoError::Database(e.to_string()))?;

        Ok(UserStatus::from(status_code).as_db_str().to_string())
    }
}
