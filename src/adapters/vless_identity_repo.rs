use diesel::{prelude::*, sqlite::SqliteConnection};

use crate::{
    db::{models::VlessIdentity, schema::vless_identity, ConnPool},
    ports::{vless_identity::VlessIdentityRepoTrait, RepoError},
};

type DbConn = diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<SqliteConnection>>;

pub struct VlessIdentityRepo {
    pool: ConnPool,
}

impl VlessIdentityRepo {
    pub fn new(pool: ConnPool) -> Self {
        Self { pool }
    }

    fn conn(&self) -> Result<DbConn, RepoError> {
        self.pool
            .get()
            .map_err(|e| RepoError::Database(e.to_string()))
    }

    fn insert_batch(&self, count: usize, conn: &mut DbConn) -> Result<(), RepoError> {
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
