//! The Biome submodule provides support for user management,
//! user credential management, private key management, and user
//! notifications.
//!
//! User Management: API for CRUD operations around managing users.
//!
//! Credential Management: API to register and authenticate a user using
//! a username and password. Not recommend for use in production.
//!
//! Private Key Management: API to store and retrieve encrypted private keys.
//!
//! User Notifications: API to create and manage user notifications.

#[cfg(feature = "biome-client")]
pub mod client;

#[cfg(feature = "biome-credentials")]
pub mod credentials;

#[cfg(feature = "biome-key-management")]
pub mod key_management;

#[cfg(feature = "oauth")]
pub mod oauth;

#[cfg(feature = "biome-profile")]
pub mod profile;

#[cfg(feature = "biome-credentials")]
pub mod refresh_tokens;

#[cfg(all(feature = "biome-credentials", feature = "diesel"))]
pub use credentials::store::diesel::DieselCredentialsStore;
#[cfg(feature = "biome-credentials")]
pub use credentials::store::memory::MemoryCredentialsStore;
#[cfg(feature = "biome-credentials")]
pub use credentials::store::CredentialsStore;

#[cfg(all(feature = "biome-key-management", feature = "diesel"))]
pub use key_management::store::diesel::DieselKeyStore;
#[cfg(feature = "biome-key-management")]
pub use key_management::store::memory::MemoryKeyStore;
#[cfg(feature = "biome-key-management")]
pub use key_management::store::KeyStore;

#[cfg(all(feature = "oauth", feature = "diesel"))]
pub use oauth::store::diesel::DieselOAuthUserSessionStore;
#[cfg(feature = "oauth")]
pub use oauth::store::memory::MemoryOAuthUserSessionStore;
#[cfg(feature = "oauth")]
pub use oauth::store::OAuthUserSessionStore;

#[cfg(all(feature = "biome-profile", feature = "diesel"))]
pub use profile::store::diesel::DieselUserProfileStore;
#[cfg(feature = "biome-profile")]
pub use profile::store::memory::MemoryUserProfileStore;
#[cfg(feature = "biome-profile")]
pub use profile::store::UserProfileStore;

#[cfg(all(feature = "biome-credentials", feature = "diesel"))]
pub use refresh_tokens::store::diesel::DieselRefreshTokenStore;
#[cfg(feature = "biome-credentials")]
pub use refresh_tokens::store::memory::MemoryRefreshTokenStore;
#[cfg(feature = "biome-credentials")]
pub use refresh_tokens::store::RefreshTokenStore;
