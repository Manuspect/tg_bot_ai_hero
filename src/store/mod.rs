#[cfg(all(feature = "store-factory", feature = "memory"))]
pub mod memory;
#[cfg(feature = "diesel")]
pub(crate) mod pool;
#[cfg(all(feature = "store-factory", feature = "postgres"))]
pub mod postgres;
#[cfg(all(feature = "store-factory", feature = "sqlite"))]
pub mod sqlite;

/// An abstract factory for creating Splinter stores backed by the same storage
#[cfg(feature = "store-factory")]
pub trait StoreFactory {
    /// Get a new `CredentialsStore`
    #[cfg(feature = "biome-credentials")]
    fn get_biome_credentials_store(&self) -> Box<dyn crate::biome::CredentialsStore>;

    /// Get a new `KeyStore`
    #[cfg(feature = "biome-key-management")]
    fn get_biome_key_store(&self) -> Box<dyn crate::biome::KeyStore>;

    /// Get a new `RefreshTokenStore`
    #[cfg(feature = "biome-credentials")]
    fn get_biome_refresh_token_store(&self) -> Box<dyn crate::biome::RefreshTokenStore>;

    /// Get a new `OAuthUserSessionStore`
    #[cfg(feature = "oauth")]
    fn get_biome_oauth_user_session_store(&self) -> Box<dyn crate::biome::OAuthUserSessionStore>;

    #[cfg(feature = "admin-service")]
    fn get_admin_service_store(&self) -> Box<dyn crate::admin::store::AdminServiceStore>;

    #[cfg(feature = "oauth")]
    fn get_oauth_inflight_request_store(
        &self,
    ) -> Box<dyn crate::oauth::store::InflightOAuthRequestStore>;

    #[cfg(feature = "registry")]
    fn get_registry_store(&self) -> Box<dyn crate::registry::RwRegistry>;

    #[cfg(feature = "authorization-handler-rbac")]
    fn get_role_based_authorization_store(
        &self,
    ) -> Box<dyn crate::rbac::store::RoleBasedAuthorizationStore>;

    #[cfg(feature = "biome-profile")]
    fn get_biome_user_profile_store(&self) -> Box<dyn crate::modules::biome::UserProfileStore>;

    fn get_stats_store(&self) -> Box<dyn crate::modules::stats::UserTokenUsageStore>;
    fn get_prefs_store(&self) -> Box<dyn crate::modules::prefs::UserPreferencesStore>;
    fn get_member_store(&self) -> Box<dyn crate::modules::admin::UserMembersStore>;
    fn get_channel_message_store(
        &self,
    ) -> Box<dyn crate::modules::parser::UserChannelMessagesStore>;

    #[cfg(feature = "node-id-store")]
    fn get_node_id_store(&self) -> Box<dyn crate::node_id::store::NodeIdStore>;

    #[cfg(feature = "service-lifecycle-store")]
    fn get_lifecycle_store(&self) -> Box<dyn crate::runtime::service::LifecycleStore + Send>;
}
