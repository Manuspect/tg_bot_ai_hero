//! Defines a basic representation of a token_usage.

#[cfg(feature = "diesel")]
pub mod diesel;
pub mod error;
pub mod memory;

use crate::modules::error::InvalidStateError;
use serde::{Deserialize, Serialize};

pub use error::TokenUsageStoreError;

/// Represents a user token_usage used to display token_usage
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenUsage {
    user_id: String,
    time: chrono::NaiveDateTime,
    tokens: i64,
}

impl TokenUsage {
    /// Returns the user_id for the token_usage
    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    /// Returns the time for the token_usage
    pub fn time(&self) -> &chrono::NaiveDateTime {
        &self.time
    }

    /// Returns the tokens for the token_usage
    pub fn tokens(&self) -> &i64 {
        &self.tokens
    }
}

/// Builder for token_usage.
///
/// user_id and subject are required
#[derive(Default)]
pub struct TokenUsageBuilder {
    user_id: Option<String>,
    time: Option<chrono::NaiveDateTime>,
    tokens: Option<i64>,
}

impl TokenUsageBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the user_id for the token_usage
    ///
    /// This is a required field for the final Profile struct
    ///
    /// # Arguments
    ///
    /// * `user_id` - a unique identifier for the user the token_usage belongs to
    pub fn with_user_id(mut self, user_id: String) -> TokenUsageBuilder {
        self.user_id = Some(user_id);
        self
    }

    /// Sets the subject for the token_usage
    ///
    /// This is a required field for the final Profile struct
    ///
    /// # Arguments
    ///
    /// * `subject` - the subject id for the account that provided the token_usage information
    pub fn with_time(mut self, time: chrono::NaiveDateTime) -> TokenUsageBuilder {
        self.time = Some(time);
        self
    }

    /// Sets the subject for the token_usage
    ///
    /// This is a required field for the final Profile struct
    ///
    /// # Arguments
    ///
    /// * `subject` - the subject id for the account that provided the token_usage information
    pub fn with_tokens(mut self, tokens: i64) -> TokenUsageBuilder {
        self.tokens = Some(tokens);
        self
    }

    /// Builds the token_usage
    ///
    /// # Errors
    ///
    /// Returns an `InvalidStateError` if `user_id` or `subject` are missing
    pub fn build(self) -> Result<TokenUsage, InvalidStateError> {
        Ok(TokenUsage {
            user_id: self.user_id.ok_or_else(|| {
                InvalidStateError::with_message(
                    "A user id is required to build a TokenUsage".into(),
                )
            })?,
            time: self.time.ok_or_else(|| {
                InvalidStateError::with_message("A time is required to build a TokenUsage".into())
            })?,
            tokens: self.tokens.ok_or_else(|| {
                InvalidStateError::with_message("A tokens is required to build a TokenUsage".into())
            })?,
        })
    }
}

/// Defines methods for CRUD operations and fetching a user’s
/// token_usage without defining a storage strategy
pub trait UserTokenUsageStore: Sync + Send {
    /// Adds a token_usage to the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `token_usage` - The token_usage to be added
    ///
    /// # Errors
    ///
    /// Returns a TokenUsageStoreError if the implementation cannot add a new
    /// token_usage.
    fn add_token_usage(&self, token_usage: TokenUsage) -> Result<(), TokenUsageStoreError>;

    fn query_usage_of_user(&self, user_id: &str) -> Result<i64, TokenUsageStoreError>;

    fn query_total_usage(&self) -> Result<i64, TokenUsageStoreError>;

    /// Clone into a boxed, dynamically dispatched store
    fn clone_box(&self) -> Box<dyn UserTokenUsageStore>;
}

impl Clone for Box<dyn UserTokenUsageStore> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl<PS> UserTokenUsageStore for Box<PS>
where
    PS: UserTokenUsageStore + ?Sized,
{
    fn add_token_usage(&self, token_usage: TokenUsage) -> Result<(), TokenUsageStoreError> {
        (**self).add_token_usage(token_usage)
    }

    fn clone_box(&self) -> Box<dyn UserTokenUsageStore> {
        (**self).clone_box()
    }

    fn query_usage_of_user(&self, user_id: &str) -> Result<i64, TokenUsageStoreError> {
        (**self).query_usage_of_user(user_id)
    }

    fn query_total_usage(&self) -> Result<i64, TokenUsageStoreError> {
        (**self).query_total_usage()
    }
}
