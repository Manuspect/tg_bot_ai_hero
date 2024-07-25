//! Defines a basic representation of a channel_messages.

#[cfg(feature = "diesel")]
pub mod diesel;
pub mod error;
pub mod memory;

use crate::modules::error::InvalidStateError;
use serde::{Deserialize, Serialize};

pub use error::ChannelMessagesStoreError;

/// Represents a user token_usage used to display token_usage
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelMessages {
    /// The ID of the message in the "copy tg channel".
    pub tg_copy_channel_message_id: Option<i32>,
    /// The ID of the message in the "copy vk channel".
    pub vk_copy_channel_message_id: Option<i32>,
    /// The ID of the message in the "copy inst channel".
    pub inst_copy_channel_message_id: Option<i32>,
    /// The ID of the corresponding message in the "paste channel".
    pub paste_channel_message_id: Option<i32>,
    /// The timestamp that indicates the insertion time.
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl ChannelMessages {
    /// Returns the tg_copy_channel_message_id for the channel_messages
    pub fn tg_copy_channel_message_id(&self) -> std::option::Option<&i32> {
        self.tg_copy_channel_message_id.as_ref()
    }

    /// Returns the vk_copy_channel_message_id for the channel_messages
    pub fn vk_copy_channel_message_id(&self) -> std::option::Option<&i32> {
        self.vk_copy_channel_message_id.as_ref()
    }

    /// Returns the inst_copy_channel_message_id for the channel_messages
    pub fn inst_copy_channel_message_id(&self) -> std::option::Option<&i32> {
        self.inst_copy_channel_message_id.as_ref()
    }

    /// Returns the paste_channel_message_id for the channel_messages
    pub fn paste_channel_message_id(&self) -> std::option::Option<&i32> {
        self.paste_channel_message_id.as_ref()
    }

    /// Returns the created_at for the channel_messages
    pub fn created_at(&self) -> &chrono::NaiveDateTime {
        &self.created_at
    }

    /// Returns the updated_at for the channel_messages
    pub fn updated_at(&self) -> &chrono::NaiveDateTime {
        &self.updated_at
    }
}

/// Builder for channel_messages.
///
/// user_id and subject are required
#[derive(Default)]
pub struct ChannelMessagesBuilder {
    /// The ID of the message in the "copy tg channel".
    pub tg_copy_channel_message_id: Option<i32>,
    /// The ID of the message in the "copy vk channel".
    pub vk_copy_channel_message_id: Option<i32>,
    /// The ID of the message in the "copy inst channel".
    pub inst_copy_channel_message_id: Option<i32>,
    /// The ID of the corresponding message in the "paste channel".
    pub paste_channel_message_id: Option<i32>,
    /// The timestamp that indicates the insertion time.
    pub created_at: Option<chrono::NaiveDateTime>,
    /// The timestamp that indicates the last update time.
    pub updated_at: Option<chrono::NaiveDateTime>,
}

impl ChannelMessagesBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the tg_copy_channel_message_id for the channel_messages
    ///
    /// This is a required field for the final ChannelMessages struct
    ///
    /// # Arguments
    ///
    /// * `tg_copy_channel_message_id` - the tg_copy_channel_message_id for the account that provided the channel_messages information
    pub fn with_tg_copy_channel_message_id(
        mut self,
        tg_copy_channel_message_id: i32,
    ) -> ChannelMessagesBuilder {
        self.tg_copy_channel_message_id = Some(tg_copy_channel_message_id);
        self
    }

    /// Sets the vk_copy_channel_message_id for the channel_messages
    /// This is a required field for the final ChannelMessages struct
    ///
    /// # Arguments
    ///
    /// * `vk_copy_channel_message_id` - the vk_copy_channel_message_id for the account that provided the channel_messages information
    pub fn with_vk_copy_channel_message_id(
        mut self,
        vk_copy_channel_message_id: i32,
    ) -> ChannelMessagesBuilder {
        self.vk_copy_channel_message_id = Some(vk_copy_channel_message_id);
        self
    }

    /// Sets the inst_copy_channel_message_id for the channel_messages
    /// This is a required field for the final ChannelMessages struct
    ///
    /// Arguments
    ///
    /// * `inst_copy_channel_message_id` - the inst_copy_channel_message_id for the account that provided the channel_messages information
    pub fn with_inst_copy_channel_message_id(
        mut self,
        inst_copy_channel_message_id: i32,
    ) -> ChannelMessagesBuilder {
        self.inst_copy_channel_message_id = Some(inst_copy_channel_message_id);
        self
    }

    /// Sets the paste_channel_message_id for the channel_messages
    /// This is a required field for the final ChannelMessages struct
    ///
    /// # Arguments
    ///
    /// * `paste_channel_message_id` - the paste_channel_message_id for the account that provided the channel_messages information
    pub fn with_paste_channel_message_id(
        mut self,
        paste_channel_message_id: i32,
    ) -> ChannelMessagesBuilder {
        self.paste_channel_message_id = Some(paste_channel_message_id);
        self
    }

    /// Sets the created_at for the channel_messages
    ///
    /// This is a required field for the final ChannelMessages struct
    ///
    /// # Arguments
    ///
    /// * `created_at` - the created_at id for the account that provided the channel_messages information
    pub fn with_created_at(mut self, created_at: chrono::NaiveDateTime) -> ChannelMessagesBuilder {
        self.created_at = Some(created_at);
        self
    }

    /// Sets the updated_at for the channel_messages
    /// This is a required field for the final ChannelMessages struct
    ///
    /// # Arguments
    ///
    /// * `updated_at` - the updated_at id for the account that provided the channel_messages information
    pub fn with_updated_at(mut self, updated_at: chrono::NaiveDateTime) -> ChannelMessagesBuilder {
        self.updated_at = Some(updated_at);
        self
    }

    /// Builds the channel_messages
    ///
    /// # Errors
    ///
    /// Returns an `InvalidStateError` if `message_id` or `created_at` are missing
    pub fn build(self) -> Result<ChannelMessages, InvalidStateError> {
        Ok(ChannelMessages {
            tg_copy_channel_message_id: self.tg_copy_channel_message_id,
            vk_copy_channel_message_id: self.vk_copy_channel_message_id,
            inst_copy_channel_message_id: self.inst_copy_channel_message_id,
            paste_channel_message_id: self.paste_channel_message_id,
            created_at: self.created_at.ok_or_else(|| {
                InvalidStateError::with_message(
                    "A created_at is required to build a ChannelMessages".into(),
                )
            })?,
            updated_at: self.updated_at.ok_or_else(|| {
                InvalidStateError::with_message(
                    "A created_at is required to build a ChannelMessages".into(),
                )
            })?,
        })
    }
}

/// Defines methods for CRUD operations and fetching a user’s
/// channel_messages without defining a storage strategy
pub trait UserChannelMessagesStore: Sync + Send {
    /// Adds a channel_message to the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `channel_message` - The channel_message to be added
    ///
    /// # Errors
    ///
    /// Returns a ChannelMessagesStoreError if the implementation cannot add a new
    /// channel_messages.
    fn add_channel_message(
        &self,
        channel_message: ChannelMessages,
    ) -> Result<(), ChannelMessagesStoreError>;

    fn delete_channel_message(&self, message_id: &i32) -> Result<(), ChannelMessagesStoreError>;

    fn get_channel_message(
        &self,
        message_id: &i32,
    ) -> Result<ChannelMessages, ChannelMessagesStoreError>;

    // This function retrieves messages by its id from the database.
    /// The function returns an error if the operation fails.
    fn get_from_db_by_tg_id(
        &mut self,
        tg_copy_channel_message_id: &i32,
    ) -> Result<ChannelMessages, ChannelMessagesStoreError>;

    /// This function removes all messages that are older than a particular
    /// timestamp. (This function is necessary not to overload the database
    /// with messages).s
    fn remove_old_messages(
        &mut self,
        timestamp: chrono::NaiveDateTime,
    ) -> Result<(), ChannelMessagesStoreError>;

    /// Clone into a boxed, dynamically dispatched store
    fn clone_box(&self) -> Box<dyn UserChannelMessagesStore>;
}

impl Clone for Box<dyn UserChannelMessagesStore> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl<PS> UserChannelMessagesStore for Box<PS>
where
    PS: UserChannelMessagesStore + ?Sized,
{
    fn clone_box(&self) -> Box<dyn UserChannelMessagesStore> {
        (**self).clone_box()
    }

    fn add_channel_message(
        &self,
        channel_message: ChannelMessages,
    ) -> Result<(), ChannelMessagesStoreError> {
        (**self).add_channel_message(channel_message)
    }

    fn delete_channel_message(&self, message_id: &i32) -> Result<(), ChannelMessagesStoreError> {
        (**self).delete_channel_message(message_id)
    }

    fn get_channel_message(
        &self,
        message_id: &i32,
    ) -> Result<ChannelMessages, ChannelMessagesStoreError> {
        (**self).get_channel_message(message_id)
    }

    fn get_from_db_by_tg_id(
        &mut self,
        tg_copy_channel_message_id: &i32,
    ) -> Result<ChannelMessages, ChannelMessagesStoreError> {
        (**self).get_from_db_by_tg_id(tg_copy_channel_message_id)
    }

    fn remove_old_messages(
        &mut self,
        timestamp: chrono::NaiveDateTime,
    ) -> Result<(), ChannelMessagesStoreError> {
        (**self).remove_old_messages(timestamp)
    }
}
