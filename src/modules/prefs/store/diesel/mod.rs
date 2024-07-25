//! Database-backed implementation of the [PreferencesStore], powered by [diesel].

pub mod models;
mod operations;

use std::sync::{Arc, RwLock};

use diesel::r2d2::{ConnectionManager, Pool};

use crate::store::pool::ConnectionPool;

use self::operations::get_value::PreferencesStoreGetPreference;

use super::{Preferences, PreferencesStoreError, UserPreferencesStore};

use models::PreferencesModel;

use operations::{set_value::PreferencesStoreAddPreference as _, UserPreferencesStoreOperations};

/// Manages creating, updating, and fetching preferences from the database
pub struct DieselPreferencesStore<C: diesel::r2d2::R2D2Connection + 'static> {
    connection_pool: ConnectionPool<C>,
}

impl<C: diesel::r2d2::R2D2Connection> DieselPreferencesStore<C> {
    /// Creates a new DieselPreferencesStore
    ///
    /// # Arguments
    ///
    ///  * `connection_pool`: connection pool to the database
    pub fn new(connection_pool: Pool<ConnectionManager<C>>) -> Self {
        DieselPreferencesStore {
            connection_pool: connection_pool.into(),
        }
    }

    /// Create a new `DieselPreferencesStore` with write exclusivity enabled.
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
impl PreferencesStore for DieselPreferencesStore<diesel::pg::PgConnection> {
    fn add_profile(&self, profile: Preference) -> Result<(), PreferencesStoreError> {
        self.connection_pool.execute_write(|connection| {
            PreferencesStoreOperations::new(connection).add_profile(profile)
        })
    }

    fn clone_box(&self) -> Box<dyn PreferencesStore> {
        Box::new(Self {
            connection_pool: self.connection_pool.clone(),
        })
    }
}

#[cfg(feature = "sqlite")]
impl UserPreferencesStore for DieselPreferencesStore<diesel::sqlite::SqliteConnection> {
    fn clone_box(&self) -> Box<dyn UserPreferencesStore> {
        Box::new(Self {
            connection_pool: self.connection_pool.clone(),
        })
    }

    fn set_value(&self, prefs: Preferences) -> Result<(), PreferencesStoreError> {
        self.connection_pool.execute_write(|connection| {
            UserPreferencesStoreOperations::new(connection).set_value(prefs)
        })
    }

    fn get_value(&self, key: &str) -> Result<Preferences, PreferencesStoreError> {
        self.connection_pool.execute_read(|connection| {
            UserPreferencesStoreOperations::new(connection).get_value(key)
        })
    }
}

impl From<PreferencesModel> for Preferences {
    fn from(user_preferences: PreferencesModel) -> Self {
        Self {
            key: user_preferences.pref_key,
            value: user_preferences.value,
        }
    }
}
