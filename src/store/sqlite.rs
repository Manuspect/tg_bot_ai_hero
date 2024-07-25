//! Implementation of a `StoreFactory` for SQLite
use anyhow::Error;
use std::sync::{Arc, RwLock};

use diesel::{
    connection::SimpleConnection,
    r2d2::{ConnectionManager, CustomizeConnection, Pool},
    sqlite::SqliteConnection,
};

#[cfg(feature = "authorization-handler-rbac")]
use crate::rbac::store::{DieselRoleBasedAuthorizationStore, RoleBasedAuthorizationStore};
use build_database::migrations::{any_pending_sqlite_migrations, run_sqlite_migrations};

use super::StoreFactory;

/// Create a SQLite connection pool.
///
/// # Arguments
///
/// * conn_str - a filename or ":memory:"
///
/// # Errors
///
/// An [InternalError] is returned if
/// * The file does not exist
/// * The pool cannot be created
/// * The database requires any pending migrations
pub fn create_sqlite_connection_pool(
    conn_str: &str,
) -> Result<Pool<ConnectionManager<SqliteConnection>>, Error> {
    if (conn_str != ":memory:") && !std::path::Path::new(&conn_str).exists() {
        return Err(anyhow!(format!(
            "Database file '{}' does not exist",
            conn_str
        )));
    }
    let connection_manager = ConnectionManager::<SqliteConnection>::new(conn_str);
    let mut pool_builder = Pool::builder()
        .connection_customizer(Box::new(ConnectionCustomizer::default()))
        .error_handler(Box::new(HandlePoolError));
    // A new database is created for each connection to the in-memory SQLite
    // implementation; to ensure that the resulting stores will operate on the same
    // database, only one connection is allowed.
    if conn_str == ":memory:" {
        pool_builder = pool_builder.max_size(1);
    }
    let pool = pool_builder
        .build(connection_manager)
        .map_err(|err| anyhow!(format!("Failed to build connection pool: {}", err)))?;
    let mut conn = pool.get().map_err(|err| anyhow!(err))?;

    if conn_str == ":memory:" {
        run_sqlite_migrations(&mut conn)?;
    } else if !any_pending_sqlite_migrations(&mut conn)? {
        return Err(anyhow!(String::from(
            "This version of splinter requires migrations that are not yet applied \
            to the database. Run `ai_hero database migrate` to apply migrations \
            before running splinterd",
        )));
    }

    Ok(pool)
}

pub fn create_sqlite_connection_pool_with_write_exclusivity(
    conn_str: &str,
) -> Result<Arc<RwLock<Pool<ConnectionManager<SqliteConnection>>>>, Error> {
    Ok(Arc::new(RwLock::new(create_sqlite_connection_pool(
        conn_str,
    )?)))
}

/// A `StoreFactory` backed by a SQLite database.
pub struct SqliteStoreFactory {
    pool: Arc<RwLock<Pool<ConnectionManager<SqliteConnection>>>>,
}

impl SqliteStoreFactory {
    /// Create a new `SqliteStoreFactory`.
    pub fn new(pool: Pool<ConnectionManager<SqliteConnection>>) -> Self {
        Self {
            pool: Arc::new(RwLock::new(pool)),
        }
    }

    /// Create a new `SqliteStoreFactory` with shared write-exclusivity.
    pub fn new_with_write_exclusivity(
        pool: Arc<RwLock<Pool<ConnectionManager<SqliteConnection>>>>,
    ) -> Self {
        Self { pool }
    }
}

impl StoreFactory for SqliteStoreFactory {
    #[cfg(feature = "biome-credentials")]
    fn get_biome_credentials_store(&self) -> Box<dyn crate::biome::CredentialsStore> {
        Box::new(
            crate::biome::DieselCredentialsStore::new_with_write_exclusivity(self.pool.clone()),
        )
    }

    #[cfg(feature = "biome-key-management")]
    fn get_biome_key_store(&self) -> Box<dyn crate::biome::KeyStore> {
        Box::new(crate::biome::DieselKeyStore::new_with_write_exclusivity(
            self.pool.clone(),
        ))
    }

    #[cfg(feature = "biome-credentials")]
    fn get_biome_refresh_token_store(&self) -> Box<dyn crate::biome::RefreshTokenStore> {
        Box::new(
            crate::biome::DieselRefreshTokenStore::new_with_write_exclusivity(self.pool.clone()),
        )
    }

    #[cfg(feature = "oauth")]
    fn get_biome_oauth_user_session_store(&self) -> Box<dyn crate::biome::OAuthUserSessionStore> {
        Box::new(
            crate::biome::DieselOAuthUserSessionStore::new_with_write_exclusivity(
                self.pool.clone(),
            ),
        )
    }

    #[cfg(feature = "admin-service")]
    fn get_admin_service_store(&self) -> Box<dyn crate::admin::store::AdminServiceStore> {
        Box::new(
            crate::admin::store::diesel::DieselAdminServiceStore::new_with_write_exclusivity(
                self.pool.clone(),
            ),
        )
    }

    #[cfg(feature = "oauth")]
    fn get_oauth_inflight_request_store(
        &self,
    ) -> Box<dyn crate::oauth::store::InflightOAuthRequestStore> {
        Box::new(
            crate::oauth::store::DieselInflightOAuthRequestStore::new_with_write_exclusivity(
                self.pool.clone(),
            ),
        )
    }

    #[cfg(feature = "registry")]
    fn get_registry_store(&self) -> Box<dyn crate::registry::RwRegistry> {
        Box::new(crate::registry::DieselRegistry::new_with_write_exclusivity(
            self.pool.clone(),
        ))
    }

    #[cfg(feature = "authorization-handler-rbac")]
    fn get_role_based_authorization_store(&self) -> Box<dyn RoleBasedAuthorizationStore> {
        Box::new(DieselRoleBasedAuthorizationStore::new_with_write_exclusivity(self.pool.clone()))
    }

    #[cfg(feature = "biome-profile")]
    fn get_biome_user_profile_store(&self) -> Box<dyn crate::modules::biome::UserProfileStore> {
        Box::new(
            crate::modules::biome::DieselUserProfileStore::new_with_write_exclusivity(
                self.pool.clone(),
            ),
        )
    }

    #[cfg(feature = "node-id-store")]
    fn get_node_id_store(&self) -> Box<dyn crate::node_id::store::NodeIdStore> {
        Box::new(
            crate::node_id::store::diesel::DieselNodeIdStore::new_with_write_exclusivity(
                self.pool.clone(),
            ),
        )
    }

    #[cfg(feature = "service-lifecycle-store")]
    fn get_lifecycle_store(&self) -> Box<dyn crate::runtime::service::LifecycleStore + Send> {
        Box::new(
            crate::runtime::service::DieselLifecycleStore::new_with_write_exclusivity(
                self.pool.clone(),
            ),
        )
    }

    fn get_stats_store(&self) -> Box<dyn crate::modules::stats::UserTokenUsageStore> {
        Box::new(
            crate::modules::stats::DieselTokenUsageStore::new_with_write_exclusivity(
                self.pool.clone(),
            ),
        )
    }

    fn get_prefs_store(&self) -> Box<dyn crate::modules::prefs::UserPreferencesStore> {
        Box::new(
            crate::modules::prefs::DieselPreferencesStore::new_with_write_exclusivity(
                self.pool.clone(),
            ),
        )
    }

    fn get_member_store(&self) -> Box<dyn crate::modules::admin::UserMembersStore> {
        Box::new(
            crate::modules::admin::DieselMembersStore::new_with_write_exclusivity(
                self.pool.clone(),
            ),
        )
    }

    fn get_channel_message_store(
        &self,
    ) -> Box<dyn crate::modules::parser::UserChannelMessagesStore> {
        Box::new(
            crate::modules::parser::DieselChannelMessagesStore::new_with_write_exclusivity(
                self.pool.clone(),
            ),
        )
    }
}

#[derive(Default, Debug)]
/// Foreign keys must be enabled on a per connection basis. This customizer will be added to the
/// SQLite pool builder and then ran against every connection returned from the pool.
pub struct ConnectionCustomizer;

impl CustomizeConnection<SqliteConnection, diesel::r2d2::Error> for ConnectionCustomizer {
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        conn.batch_execute(
            r#"
            PRAGMA busy_timeout = 2000;
            PRAGMA foreign_keys = ON;
            "#,
        )
        .map_err(diesel::r2d2::Error::QueryError)
    }
}

#[derive(Debug)]
struct HandlePoolError;

impl diesel::r2d2::HandleError<diesel::r2d2::Error> for HandlePoolError {
    fn handle_error(&self, error: diesel::r2d2::Error) {
        // Ignore the logging of "database is locked" error when submitting the pragma to the new
        // connection. The connection will be retried by the connection manager.
        if &error.to_string() != "database is locked" {
            log::error!("{}", error);
        }
    }
}
