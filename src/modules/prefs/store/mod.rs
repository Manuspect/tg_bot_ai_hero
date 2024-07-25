//! Defines a basic representation of a preferences.

#[cfg(feature = "diesel")]
pub mod diesel;
pub mod error;
pub mod memory;

use crate::modules::error::InvalidStateError;
use serde::{Deserialize, Serialize};

pub use error::PreferencesStoreError;

/// Represents a user token_usage used to display token_usage
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Preferences {
    key: String,
    value: Option<String>,
}

impl Preferences {
    /// Returns the key for the preferences
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Returns the value for the preferences
    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }
}

/// Builder for preferences.
///
/// user_id and subject are required
#[derive(Default)]
pub struct PreferencesBuilder {
    key: Option<String>,
    value: Option<String>,
}

impl PreferencesBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the user_id for the preferences
    ///
    /// This is a required field for the final Preferences struct
    ///
    /// # Arguments
    ///
    /// * `key` - a unique identifier for the user the preferences belongs to
    pub fn with_key(mut self, key: String) -> PreferencesBuilder {
        self.key = Some(key);
        self
    }

    /// Sets the value for the preferences
    ///
    /// This is a required field for the final Preferences struct
    ///
    /// # Arguments
    ///
    /// * `value` - the value id for the account that provided the preferences information
    pub fn with_value(mut self, value: String) -> PreferencesBuilder {
        self.value = Some(value);
        self
    }

    /// Builds the preferences
    ///
    /// # Errors
    ///
    /// Returns an `InvalidStateError` if `key` is missing
    pub fn build(self) -> Result<Preferences, InvalidStateError> {
        Ok(Preferences {
            key: self.key.ok_or_else(|| {
                InvalidStateError::with_message(
                    "A user id is required to build a Preferences".into(),
                )
            })?,
            value: self.value,
        })
    }
}

/// Defines methods for CRUD operations and fetching a user’s
/// preferences without defining a storage strategy
pub trait UserPreferencesStore: Sync + Send {
    /// Adds a prefs to the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `prefs` - The prefs to be added
    ///
    /// # Errors
    ///
    /// Returns a PreferencesStoreError if the implementation cannot add a new
    /// preferences.
    fn set_value(&self, prefs: Preferences) -> Result<(), PreferencesStoreError>;

    fn get_value(&self, key: &str) -> Result<Preferences, PreferencesStoreError>;

    /// Clone into a boxed, dynamically dispatched store
    fn clone_box(&self) -> Box<dyn UserPreferencesStore>;
}

impl Clone for Box<dyn UserPreferencesStore> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl<PS> UserPreferencesStore for Box<PS>
where
    PS: UserPreferencesStore + ?Sized,
{
    fn clone_box(&self) -> Box<dyn UserPreferencesStore> {
        (**self).clone_box()
    }

    fn set_value(&self, prefs: Preferences) -> Result<(), PreferencesStoreError> {
        (**self).set_value(prefs)
    }

    fn get_value(&self, key: &str) -> Result<Preferences, PreferencesStoreError> {
        (**self).get_value(key)
    }
}
