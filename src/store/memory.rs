//! Implementation of a `StoreFactory` for in memory

use anyhow::Error;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    sqlite::SqliteConnection,
};

#[cfg(feature = "oauth")]
use crate::biome::MemoryOAuthUserSessionStore;
#[cfg(feature = "biome-credentials")]
use crate::biome::{
    CredentialsStore, MemoryCredentialsStore, MemoryRefreshTokenStore, RefreshTokenStore,
};
#[cfg(feature = "biome-key-management")]
use crate::biome::{KeyStore, MemoryKeyStore};
#[cfg(feature = "biome-profile")]
use crate::modules::biome::{MemoryUserProfileStore, UserProfileStore};
use crate::modules::{
    admin::MemoryMembersStore, parser::MemoryChannelMessagesStore, prefs::MemoryPreferencesStore,
    stats::MemoryTokenUsageStore,
};
#[cfg(feature = "oauth")]
use crate::oauth::store::MemoryInflightOAuthRequestStore;

use super::sqlite::ConnectionCustomizer;
use super::StoreFactory;

/// A `StoryFactory` backed by memory.
pub struct MemoryStoreFactory {
    #[cfg(feature = "biome-credentials")]
    biome_credentials_store: MemoryCredentialsStore,
    #[cfg(feature = "biome-key-management")]
    biome_key_store: MemoryKeyStore,
    #[cfg(feature = "biome-credentials")]
    biome_refresh_token_store: MemoryRefreshTokenStore,
    #[cfg(feature = "oauth")]
    biome_oauth_user_session_store: MemoryOAuthUserSessionStore,
    #[cfg(feature = "oauth")]
    inflight_request_store: MemoryInflightOAuthRequestStore,
    #[cfg(feature = "biome-profile")]
    biome_profile_store: MemoryUserProfileStore,
    token_usage_store: MemoryTokenUsageStore,
    preferences_store: MemoryPreferencesStore,
    members_store: MemoryMembersStore,
    channel_message_store: MemoryChannelMessagesStore,
    // to be used for sqlite in memory implementations
    pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl MemoryStoreFactory {
    pub fn new() -> Result<Self, Error> {
        #[cfg(feature = "biome-credentials")]
        let biome_credentials_store = MemoryCredentialsStore::new();

        #[cfg(all(feature = "biome-key-management", feature = "biome-credentials"))]
        let biome_key_store = MemoryKeyStore::new(biome_credentials_store.clone());
        #[cfg(all(feature = "biome-key-management", not(feature = "biome-credentials")))]
        let biome_key_store = MemoryKeyStore::new();

        #[cfg(feature = "oauth")]
        let biome_oauth_user_session_store = MemoryOAuthUserSessionStore::new();

        #[cfg(feature = "oauth")]
        let inflight_request_store = MemoryInflightOAuthRequestStore::new();

        #[cfg(feature = "biome-profile")]
        let biome_profile_store = MemoryUserProfileStore::new();
        let token_usage_store = MemoryTokenUsageStore::new();
        let preferences_store = MemoryPreferencesStore::new();
        let members_store = MemoryMembersStore::new();
        let channel_message_store = MemoryChannelMessagesStore::new();

        let connection_manager = ConnectionManager::<SqliteConnection>::new(":memory:");
        let pool = Pool::builder()
            .max_size(1)
            .connection_customizer(Box::new(ConnectionCustomizer::default()))
            .build(connection_manager)
            .map_err(|err| anyhow!(Box::new(err)))?;

        build_database::migrations::run_sqlite_migrations(
            &mut *pool.get().map_err(|err| anyhow!(Box::new(err)))?,
        )
        .map_err(|err| anyhow!(Box::new(err)))?;

        Ok(Self {
            #[cfg(feature = "biome-credentials")]
            biome_credentials_store,
            #[cfg(feature = "biome-key-management")]
            biome_key_store,
            #[cfg(feature = "biome-credentials")]
            biome_refresh_token_store: MemoryRefreshTokenStore::new(),
            #[cfg(feature = "oauth")]
            biome_oauth_user_session_store,
            #[cfg(feature = "oauth")]
            inflight_request_store,
            #[cfg(feature = "biome-profile")]
            biome_profile_store,
            token_usage_store,
            preferences_store,
            members_store,
            channel_message_store,
            pool,
        })
    }
}

impl StoreFactory for MemoryStoreFactory {
    #[cfg(feature = "biome-credentials")]
    fn get_biome_credentials_store(&self) -> Box<dyn CredentialsStore> {
        Box::new(self.biome_credentials_store.clone())
    }

    #[cfg(feature = "biome-key-management")]
    fn get_biome_key_store(&self) -> Box<dyn KeyStore> {
        Box::new(self.biome_key_store.clone())
    }

    #[cfg(feature = "biome-credentials")]
    fn get_biome_refresh_token_store(&self) -> Box<dyn RefreshTokenStore> {
        Box::new(self.biome_refresh_token_store.clone())
    }

    #[cfg(feature = "oauth")]
    fn get_biome_oauth_user_session_store(&self) -> Box<dyn crate::biome::OAuthUserSessionStore> {
        Box::new(self.biome_oauth_user_session_store.clone())
    }

    #[cfg(feature = "admin-service")]
    fn get_admin_service_store(&self) -> Box<dyn crate::admin::store::AdminServiceStore> {
        Box::new(crate::admin::store::diesel::DieselAdminServiceStore::new(
            self.pool.clone(),
        ))
    }

    #[cfg(feature = "oauth")]
    fn get_oauth_inflight_request_store(
        &self,
    ) -> Box<dyn crate::oauth::store::InflightOAuthRequestStore> {
        Box::new(self.inflight_request_store.clone())
    }

    #[cfg(feature = "registry")]
    fn get_registry_store(&self) -> Box<dyn crate::registry::RwRegistry> {
        Box::new(crate::registry::DieselRegistry::new(self.pool.clone()))
    }

    #[cfg(feature = "authorization-handler-rbac")]
    fn get_role_based_authorization_store(
        &self,
    ) -> Box<dyn crate::rbac::store::RoleBasedAuthorizationStore> {
        Box::new(crate::rbac::store::DieselRoleBasedAuthorizationStore::new(
            self.pool.clone(),
        ))
    }

    #[cfg(feature = "biome-profile")]
    fn get_biome_user_profile_store(&self) -> Box<dyn UserProfileStore> {
        Box::new(self.biome_profile_store.clone())
    }

    #[cfg(feature = "node-id-store")]
    fn get_node_id_store(&self) -> Box<dyn crate::node_id::store::NodeIdStore> {
        Box::new(crate::node_id::store::diesel::DieselNodeIdStore::new(
            self.pool.clone(),
        ))
    }

    #[cfg(feature = "service-lifecycle-store")]
    fn get_lifecycle_store(&self) -> Box<dyn crate::runtime::service::LifecycleStore + Send> {
        Box::new(crate::runtime::service::DieselLifecycleStore::new(
            self.pool.clone(),
        ))
    }

    fn get_stats_store(&self) -> Box<dyn crate::modules::stats::UserTokenUsageStore> {
        Box::new(self.token_usage_store.clone())
    }

    fn get_prefs_store(&self) -> Box<dyn crate::modules::prefs::UserPreferencesStore> {
        Box::new(self.preferences_store.clone())
    }

    fn get_member_store(&self) -> Box<dyn crate::modules::admin::UserMembersStore> {
        Box::new(self.members_store.clone())
    }

    fn get_channel_message_store(
        &self,
    ) -> Box<dyn crate::modules::parser::UserChannelMessagesStore> {
        Box::new(self.channel_message_store.clone())
    }
}
