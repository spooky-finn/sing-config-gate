#![allow(dead_code)]

use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
    sqlite::SqliteConnection,
};

use crate::{
    db::{
        enums::UserStatus,
        models::{User, VlessIdentity},
        schema::{user, vless_identity},
    },
    ports::{user::UserRepoTrait, vless_identity::VlessIdentityRepoTrait, RepoError},
};

pub type ConnPool = Pool<ConnectionManager<SqliteConnection>>;

pub struct UserRepo {
    pub pool: ConnPool,
}

impl UserRepo {
    pub fn new(pool: ConnPool) -> Self {
        Self { pool }
    }

    pub fn conn(
        &self,
    ) -> Result<diesel::r2d2::PooledConnection<ConnectionManager<SqliteConnection>>, RepoError>
    {
        self.pool
            .get()
            .map_err(|e| RepoError::Database(e.to_string()))
    }
}

pub struct VlessIdentityRepo {
    pool: ConnPool,
}

impl VlessIdentityRepo {
    pub fn new(pool: ConnPool) -> Self {
        Self { pool }
    }

    pub fn conn(
        &self,
    ) -> Result<diesel::r2d2::PooledConnection<ConnectionManager<SqliteConnection>>, RepoError>
    {
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
}

impl VlessIdentityRepoTrait for VlessIdentityRepo {
    fn assign(&self, user_id: i64) -> Result<(), RepoError> {
        let mut conn = self.conn()?;

        // Find the first unassigned UUID (user_id = 0)
        let target_uuid = self.get_available_identity().or_else(|_| {
            self.insert_batch(10, &mut conn)?;
            self.get_available_identity()
        })?;

        // Assign it to the user
        diesel::update(vless_identity::table.filter(vless_identity::uuid.eq(&target_uuid)))
            .set(vless_identity::user_id.eq(&user_id))
            .execute(&mut conn)
            .map_err(|e| RepoError::Database(e.to_string()))?;

        Ok(())
    }

    fn get(&self, uuid: &str) -> Result<Option<VlessIdentity>, RepoError> {
        let mut conn = self.conn()?;

        let result = vless_identity::table
            .filter(vless_identity::uuid.eq(&uuid))
            .first::<VlessIdentity>(&mut conn)
            .optional()
            .map_err(|e| RepoError::Database(e.to_string()))?;

        Ok(result)
    }

    fn insert(&self, new_vless_identity: &VlessIdentity) -> Result<(), RepoError> {
        let mut conn = self.conn()?;

        diesel::insert_into(vless_identity::table)
            .values(new_vless_identity)
            .execute(&mut conn)
            .map_err(|e| RepoError::Database(e.to_string()))?;

        Ok(())
    }

    fn delete(&self, uuid: &str) -> Result<(), RepoError> {
        let mut conn = self.conn()?;

        diesel::delete(vless_identity::table.filter(vless_identity::uuid.eq(&uuid)))
            .execute(&mut conn)
            .map_err(|e| RepoError::Database(e.to_string()))?;

        Ok(())
    }

    fn get_by_user_id(&self, user_id: i64) -> Result<VlessIdentity, RepoError> {
        let mut conn = self.conn()?;

        let results = vless_identity::table
            .filter(vless_identity::user_id.eq(&user_id))
            .first::<VlessIdentity>(&mut conn)
            .map_err(|e| RepoError::Database(e.to_string()))?;

        Ok(results)
    }

    fn get_all(&self) -> Result<Vec<VlessIdentity>, RepoError> {
        let mut conn = self.conn()?;

        let result = vless_identity::table
            .load::<VlessIdentity>(&mut conn)
            .map_err(|e| RepoError::Database(e.to_string()))?;

        Ok(result)
    }
}

impl VlessIdentityRepo {
    fn insert_batch(&self, count: usize, conn: &mut SqliteConnection) -> Result<(), RepoError> {
        let identities: Vec<VlessIdentity> = (0..count)
            .map(|_| VlessIdentity {
                uuid: uuid::Uuid::new_v4().to_string(),
                user_id: None,
            })
            .collect();

        diesel::insert_into(vless_identity::table)
            .values(&identities)
            .execute(conn)
            .map_err(|e| RepoError::Database(e.to_string()))?;

        Ok(())
    }

    fn get_available_identity(&self) -> Result<String, RepoError> {
        let mut conn = self.conn()?;

        vless_identity::table
            .filter(vless_identity::user_id.is_null())
            .order(vless_identity::uuid.asc())
            .select(vless_identity::uuid)
            .first::<String>(&mut conn)
            .map_err(|e| match e {
                diesel::NotFound => RepoError::NotFound(e.to_string()),
                _ => RepoError::Database(e.to_string()),
            })
    }
}
