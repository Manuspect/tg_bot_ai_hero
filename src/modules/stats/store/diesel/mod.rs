//! Database-backed implementation of the [TokenUsageStore], powered by [diesel].

pub mod models;
mod operations;

use std::sync::{Arc, RwLock};

use diesel::r2d2::{ConnectionManager, Pool};

use crate::store::pool::ConnectionPool;

use self::operations::get_token_usage::UserTokenUsageStoreGetTokenUsage;

use super::{TokenUsage, TokenUsageStoreError, UserTokenUsageStore};

use models::TokenUsageModel;

use operations::{
    add_token_usage::TokenUsageStoreAddTokenUsage as _, UserTokenUsageStoreOperations,
};

/// Manages creating, updating, and fetching token_usages from the database
pub struct DieselTokenUsageStore<C: diesel::r2d2::R2D2Connection + 'static> {
    connection_pool: ConnectionPool<C>,
}

impl<C: diesel::r2d2::R2D2Connection> DieselTokenUsageStore<C> {
    /// Creates a new DieselTokenUsageStore
    ///
    /// # Arguments
    ///
    ///  * `connection_pool`: connection pool to the database
    pub fn new(connection_pool: Pool<ConnectionManager<C>>) -> Self {
        DieselTokenUsageStore {
            connection_pool: connection_pool.into(),
        }
    }

    /// Create a new `DieselTokenUsageStore` with write exclusivity enabled.
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
impl TokenUsageStore for DieselTokenUsageStore<diesel::pg::PgConnection> {
    fn add_profile(&self, profile: TokenUsage) -> Result<(), TokenUsageStoreError> {
        self.connection_pool.execute_write(|connection| {
            TokenUsageStoreOperations::new(connection).add_profile(profile)
        })
    }

    // fn update_profile(&self, profile: TokenUsage) -> Result<(), TokenUsageStoreError> {
    //     self.connection_pool.execute_write(|connection| {
    //         TokenUsageStoreOperations::new(connection).update_profile(profile)
    //     })
    // }

    // fn remove_profile(&self, user_id: &str) -> Result<(), TokenUsageStoreError> {
    //     self.connection_pool.execute_write(|connection| {
    //         TokenUsageStoreOperations::new(connection).remove_profile(user_id)
    //     })
    // }

    // fn get_profile(&self, user_id: &str) -> Result<TokenUsage, TokenUsageStoreError> {
    //     self.connection_pool.execute_read(|connection| {
    //         TokenUsageStoreOperations::new(connection).get_profile(user_id)
    //     })
    // }

    // fn list_profiles(&self) -> Result<Option<Vec<TokenUsage>>, TokenUsageStoreError> {
    //     self.connection_pool
    //         .execute_read(|connection| TokenUsageStoreOperations::new(connection).list_profiles())
    // }

    fn clone_box(&self) -> Box<dyn TokenUsageStore> {
        Box::new(Self {
            connection_pool: self.connection_pool.clone(),
        })
    }
}

#[cfg(feature = "sqlite")]
impl UserTokenUsageStore for DieselTokenUsageStore<diesel::sqlite::SqliteConnection> {
    fn add_token_usage(&self, token_usage: super::TokenUsage) -> Result<(), TokenUsageStoreError> {
        self.connection_pool.execute_write(|connection| {
            UserTokenUsageStoreOperations::new(connection).add_token_usage(token_usage)
        })
    }

    fn clone_box(&self) -> Box<dyn UserTokenUsageStore> {
        Box::new(Self {
            connection_pool: self.connection_pool.clone(),
        })
    }

    fn query_usage_of_user(&self, user_id: &str) -> Result<i64, TokenUsageStoreError> {
        self.connection_pool.execute_read(|connection| {
            UserTokenUsageStoreOperations::new(connection).query_usage_of_user(user_id)
        })
    }

    fn query_total_usage(&self) -> Result<i64, TokenUsageStoreError> {
        self.connection_pool.execute_read(|connection| {
            UserTokenUsageStoreOperations::new(connection).query_total_usage()
        })
    }
}

impl From<TokenUsageModel> for TokenUsage {
    fn from(user_token_usage: TokenUsageModel) -> Self {
        Self {
            user_id: user_token_usage.user_id,
            time: user_token_usage.time,
            tokens: user_token_usage.tokens,
        }
    }
}
