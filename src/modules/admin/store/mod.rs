//! Defines a basic representation of a members.

#[cfg(feature = "diesel")]
pub mod diesel;
pub mod error;
pub mod memory;

use crate::modules::error::InvalidStateError;
use serde::{Deserialize, Serialize};

pub use error::MembersStoreError;

/// Represents a user token_usage used to display token_usage
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Members {
    pub username: String,
    pub disabled: i32,
    pub created_at: chrono::NaiveDateTime,
}

impl Members {
    /// Returns the username for the members
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Returns the disabled for the members
    pub fn disabled(&self) -> &i32 {
        &self.disabled
    }

    /// Returns the created_at for the members
    pub fn created_at(&self) -> &chrono::NaiveDateTime {
        &self.created_at
    }
}

/// Builder for members.
///
/// user_id and subject are required
#[derive(Default)]
pub struct MembersBuilder {
    pub username: Option<String>,
    pub disabled: Option<i32>,
    pub created_at: Option<chrono::NaiveDateTime>,
}

impl MembersBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the user_id for the members
    ///
    /// This is a required field for the final Members struct
    ///
    /// # Arguments
    ///
    /// * `key` - a unique identifier for the user the members belongs to
    pub fn with_username(mut self, username: String) -> MembersBuilder {
        self.username = Some(username);
        self
    }

    /// Sets the disabled for the members
    ///
    /// This is a required field for the final Members struct
    ///
    /// # Arguments
    ///
    /// * `disabled` - the disabled id for the account that provided the members information
    pub fn with_disabled(mut self, disabled: i32) -> MembersBuilder {
        self.disabled = Some(disabled);
        self
    }

    /// Sets the created_at for the members
    ///
    /// This is a required field for the final Members struct
    ///
    /// # Arguments
    ///
    /// * `created_at` - the created_at id for the account that provided the members information
    pub fn with_created_at(mut self, created_at: chrono::NaiveDateTime) -> MembersBuilder {
        self.created_at = Some(created_at);
        self
    }

    /// Builds the members
    ///
    /// # Errors
    ///
    /// Returns an `InvalidStateError` if `username` or `created_at` are missing
    pub fn build(self) -> Result<Members, InvalidStateError> {
        Ok(Members {
            username: self.username.ok_or_else(|| {
                InvalidStateError::with_message("A username is required to build a Members".into())
            })?,
            disabled: self.disabled.ok_or_else(|| {
                InvalidStateError::with_message("A disabled is required to build a Members".into())
            })?,
            created_at: self.created_at.ok_or_else(|| {
                InvalidStateError::with_message(
                    "A created_at is required to build a Members".into(),
                )
            })?,
        })
    }
}

/// Defines methods for CRUD operations and fetching a user’s
/// members without defining a storage strategy
pub trait UserMembersStore: Sync + Send {
    /// Adds a member to the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `member` - The member to be added
    ///
    /// # Errors
    ///
    /// Returns a MembersStoreError if the implementation cannot add a new
    /// members.
    fn add_member(&self, member: Members) -> Result<(), MembersStoreError>;

    fn delete_member(&self, username: &str) -> Result<(), MembersStoreError>;

    fn get_member(&self, username: &str) -> Result<Members, MembersStoreError>;

    /// Clone into a boxed, dynamically dispatched store
    fn clone_box(&self) -> Box<dyn UserMembersStore>;
}

impl Clone for Box<dyn UserMembersStore> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl<PS> UserMembersStore for Box<PS>
where
    PS: UserMembersStore + ?Sized,
{
    fn clone_box(&self) -> Box<dyn UserMembersStore> {
        (**self).clone_box()
    }

    fn add_member(&self, member: Members) -> Result<(), MembersStoreError> {
        (**self).add_member(member)
    }

    fn delete_member(&self, username: &str) -> Result<(), MembersStoreError> {
        (**self).delete_member(username)
    }

    fn get_member(&self, username: &str) -> Result<Members, MembersStoreError> {
        (**self).get_member(username)
    }
}
