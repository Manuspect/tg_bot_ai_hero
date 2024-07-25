//! Database-backed implementation of the [MembersStore], powered by [diesel].

pub mod models;
mod operations;

use std::sync::{Arc, RwLock};

use diesel::r2d2::{ConnectionManager, Pool};

use crate::store::pool::ConnectionPool;

use super::{Members, MembersStoreError, UserMembersStore};

use models::MembersModel;

use operations::{
    add_member::MembersStoreAddMember, delete_member::MembersStoreDeleteMember,
    get_member::MembersStoreGetMember, UserMembersStoreOperations,
};

/// Manages creating, updating, and fetching members from the database
pub struct DieselMembersStore<C: diesel::r2d2::R2D2Connection + 'static> {
    connection_pool: ConnectionPool<C>,
}

impl<C: diesel::r2d2::R2D2Connection> DieselMembersStore<C> {
    /// Creates a new DieselMembersStore
    ///
    /// # Arguments
    ///
    ///  * `connection_pool`: connection pool to the database
    pub fn new(connection_pool: Pool<ConnectionManager<C>>) -> Self {
        DieselMembersStore {
            connection_pool: connection_pool.into(),
        }
    }

    /// Create a new `DieselMembersStore` with write exclusivity enabled.
    ///
    /// Write exclusivity is enforced by providing a connection pool that is wrapped in a
    /// [`RwLock`]. This ensures that there may be only one writer, but many readers.
    ///
    /// # Arguments
    ///
    ///  * `connection_pool`: read-write lock-guarded connection pool for the database
    pub fn new_with_write_exclusivity(
        connection_pool: Arc<RwLock<Pool<ConnectionManager<C>>>>,
    ) -> Self {
        Self {
            connection_pool: connection_pool.into(),
        }
    }
}

#[cfg(feature = "postgres")]
impl MembersStore for DieselMembersStore<diesel::pg::PgConnection> {
    fn add_profile(&self, profile: Member) -> Result<(), MembersStoreError> {
        self.connection_pool.execute_write(|connection| {
            MembersStoreOperations::new(connection).add_profile(profile)
        })
    }

    fn clone_box(&self) -> Box<dyn MembersStore> {
        Box::new(Self {
            connection_pool: self.connection_pool.clone(),
        })
    }
}

#[cfg(feature = "sqlite")]
impl UserMembersStore for DieselMembersStore<diesel::sqlite::SqliteConnection> {
    fn clone_box(&self) -> Box<dyn UserMembersStore> {
        Box::new(Self {
            connection_pool: self.connection_pool.clone(),
        })
    }

    fn add_member(&self, member: Members) -> Result<(), MembersStoreError> {
        self.connection_pool
            .execute_write(|connection: &mut diesel::prelude::SqliteConnection| {
                UserMembersStoreOperations::new(connection).add_member(member)
            })
    }

    fn delete_member(&self, username: &str) -> Result<(), MembersStoreError> {
        self.connection_pool.execute_write(|connection| {
            UserMembersStoreOperations::new(connection).delete_member(username)
        })
    }

    fn get_member(&self, username: &str) -> Result<Members, MembersStoreError> {
        self.connection_pool.execute_read(|connection| {
            UserMembersStoreOperations::new(connection).get_member(username)
        })
    }
}

impl From<MembersModel> for Members {
    fn from(user_members: MembersModel) -> Self {
        Self {
            username: user_members.username,
            disabled: user_members.disabled,
            created_at: user_members.created_at,
        }
    }
}
