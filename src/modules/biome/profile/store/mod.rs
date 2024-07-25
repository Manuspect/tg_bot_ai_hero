//! Defines a basic representation of a profile.

#[cfg(feature = "diesel")]
pub(in crate::modules::biome) mod diesel;
pub mod error;
pub(in crate::modules::biome) mod memory;

use crate::modules::error::InvalidStateError;
use serde::{Deserialize, Serialize};

pub use error::UserProfileStoreError;

/// Represents a user profile used to display user information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Profile {
    user_id: Option<String>,
    subject: String,
    name: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
    email: Option<String>,
    picture: Option<String>,
}

impl Profile {
    /// Returns the user_id for the profile
    pub fn user_id(&self) -> Option<&str> {
        self.user_id.as_deref()
    }

    /// Returns the subject for the profile
    pub fn subject(&self) -> &str {
        &self.subject
    }

    /// Returns the name for the profile
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns the given name for the profile
    pub fn given_name(&self) -> Option<&str> {
        self.given_name.as_deref()
    }

    /// Returns the family name for the profile
    pub fn family_name(&self) -> Option<&str> {
        self.family_name.as_deref()
    }

    /// Returns the email for the profile
    pub fn email(&self) -> Option<&str> {
        self.email.as_deref()
    }

    /// Returns the picture for the profile
    pub fn picture(&self) -> Option<&str> {
        self.picture.as_deref()
    }
}

/// Builder for profile.
///
/// user_id and subject are required
#[derive(Default)]
pub struct ProfileBuilder {
    user_id: Option<String>,
    subject: Option<String>,
    name: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
    email: Option<String>,
    picture: Option<String>,
}

impl ProfileBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the user_id for the profile
    ///
    /// This is a required field for the final Profile struct
    ///
    /// # Arguments
    ///
    /// * `user_id` - a unique identifier for the user the profile belongs to
    pub fn with_user_id(mut self, user_id: Option<String>) -> ProfileBuilder {
        self.user_id = user_id;
        self
    }

    /// Sets the subject for the profile
    ///
    /// This is a required field for the final Profile struct
    ///
    /// # Arguments
    ///
    /// * `subject` - the subject id for the account that provided the profile information
    pub fn with_subject(mut self, subject: String) -> ProfileBuilder {
        self.subject = Some(subject);
        self
    }

    /// Sets the name for the profile
    ///
    /// # Arguments
    ///
    /// * `name` - the name of the user the profile belongs to
    pub fn with_name(mut self, name: Option<String>) -> ProfileBuilder {
        self.name = name;
        self
    }

    /// Sets the given name for the profile
    ///
    /// # Arguments
    ///
    /// * `given_name` - the given name of the user the profile belongs to
    pub fn with_given_name(mut self, given_name: Option<String>) -> ProfileBuilder {
        self.given_name = given_name;
        self
    }

    /// Sets the family name for the profile
    ///
    /// # Arguments
    ///
    /// * `family_name` - the family name of the user the profile belongs to
    pub fn with_family_name(mut self, family_name: Option<String>) -> ProfileBuilder {
        self.family_name = family_name;
        self
    }

    /// Sets the email for the profile
    ///
    /// # Arguments
    ///
    /// * `email` - the user's email address associated with the account
    pub fn with_email(mut self, email: Option<String>) -> ProfileBuilder {
        self.email = email;
        self
    }

    /// Sets the picture for the profile
    ///
    /// # Arguments
    ///
    /// * `picture` - the user's profile picture associated with the account
    pub fn with_picture(mut self, picture: Option<String>) -> ProfileBuilder {
        self.picture = picture;
        self
    }

    /// Builds the profile
    ///
    /// # Errors
    ///
    /// Returns an `InvalidStateError` if `user_id` or `subject` are missing
    pub fn build(self) -> Result<Profile, InvalidStateError> {
        Ok(Profile {
            user_id: self.user_id,
            subject: self.subject.ok_or_else(|| {
                InvalidStateError::with_message("A subject is required to build a Profile".into())
            })?,
            name: self.name,
            given_name: self.given_name,
            family_name: self.family_name,
            email: self.email,
            picture: self.picture,
        })
    }
}

/// Defines methods for CRUD operations and fetching a user’s
/// profile without defining a storage strategy
pub trait UserProfileStore: Sync + Send {
    /// Adds a profile to the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `profile` - The profile to be added
    ///
    /// # Errors
    ///
    /// Returns a UserProfileStoreError if the implementation cannot add a new
    /// profile.
    fn add_profile(&self, profile: Profile) -> Result<(), UserProfileStoreError>;

    /// Replaces a profile for a user in the underlying storage with a new profile.
    ///
    /// #Arguments
    ///
    ///  * `profile` - The profile to be added
    ///
    /// # Errors
    ///
    /// Returns a UserProfileStoreError if the implementation cannot update profile
    /// or if the specified profile does not exist.
    fn update_profile(&self, profile: Profile) -> Result<(), UserProfileStoreError>;

    /// Removes a profile from the underlying storage.
    ///
    /// # Arguments
    ///
    ///  * `user_id`: The unique identifier of the user the profile belongs to
    ///
    /// # Errors
    ///
    /// Returns a UserProfileStoreError if the implementation cannot remove the
    /// profile or if a profile with the specified `user_id` does not exist.
    fn remove_profile(&self, user_id: &str) -> Result<(), UserProfileStoreError>;

    /// Fetches a profile from the underlying storage.
    ///
    /// # Arguments
    ///
    ///  * `user_id` - The unique identifier of the user the profile belongs to
    ///
    /// # Errors
    ///
    /// Returns a UserProfileStoreError if the implementation cannot retrieve the
    /// profile or if a profile with the specified `user_id` does not exist.
    fn get_profile(&self, user_id: &str) -> Result<Profile, UserProfileStoreError>;

    /// List all profiles from the underlying storage.
    ///
    /// # Errors
    ///
    /// Returns a UserProfileStoreError if implementation cannot fetch the stored
    /// profiles.
    fn list_profiles(&self) -> Result<Option<Vec<Profile>>, UserProfileStoreError>;

    /// Clone into a boxed, dynamically dispatched store
    fn clone_box(&self) -> Box<dyn UserProfileStore>;
}

impl Clone for Box<dyn UserProfileStore> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl<PS> UserProfileStore for Box<PS>
where
    PS: UserProfileStore + ?Sized,
{
    fn add_profile(&self, profile: Profile) -> Result<(), UserProfileStoreError> {
        (**self).add_profile(profile)
    }

    fn update_profile(&self, profile: Profile) -> Result<(), UserProfileStoreError> {
        (**self).update_profile(profile)
    }

    fn remove_profile(&self, user_id: &str) -> Result<(), UserProfileStoreError> {
        (**self).remove_profile(user_id)
    }

    fn get_profile(&self, user_id: &str) -> Result<Profile, UserProfileStoreError> {
        (**self).get_profile(user_id)
    }

    fn list_profiles(&self) -> Result<Option<Vec<Profile>>, UserProfileStoreError> {
        (**self).list_profiles()
    }

    fn clone_box(&self) -> Box<dyn UserProfileStore> {
        (**self).clone_box()
    }
}
