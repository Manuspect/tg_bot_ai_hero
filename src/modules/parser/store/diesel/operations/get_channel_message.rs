use crate::{
    modules::{
        error::{InternalError, InvalidArgumentError},
        parser::store::{
            diesel::models::ChannelMessagesModel, ChannelMessages, ChannelMessagesStoreError,
        },
    },
    schema::channel_messages,
};

use super::UserChannelMessagesStoreOperations;

use diesel::result::Error::NotFound;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

pub trait ChannelMessagesStoreGetMessages {
    fn get_channel_message(
        &mut self,
        channel_message: &i32,
    ) -> Result<ChannelMessages, ChannelMessagesStoreError>;

    fn get_from_db_by_tg_id(
        &mut self,
        tg_copy_channel_message_id: &i32,
    ) -> Result<ChannelMessages, ChannelMessagesStoreError>;
}

#[cfg(feature = "sqlite")]
impl<'a> ChannelMessagesStoreGetMessages
    for UserChannelMessagesStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn get_channel_message(
        &mut self,
        message_id: &i32,
    ) -> Result<ChannelMessages, ChannelMessagesStoreError> {
        let channel_message = channel_messages::table
            .filter(channel_messages::message_id.eq(message_id))
            .first::<ChannelMessagesModel>(self.conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| {
                ChannelMessagesStoreError::Internal(InternalError::with_message(format!(
                    "Failed check for existing message_id {}",
                    err
                )))
            })?
            .ok_or_else(|| {
                ChannelMessagesStoreError::InvalidArgument(InvalidArgumentError::new(
                    "message_id".to_string(),
                    "A channel_messages for the given message_id does not exist".to_string(),
                ))
            })?;
        Ok(ChannelMessages::from(channel_message))
    }

    /// This function retrieves messages by its id from the database.
    /// The function returns an error if the operation fails.
    fn get_from_db_by_tg_id(
        &mut self,
        tg_copy_channel_message_id: &i32,
    ) -> Result<ChannelMessages, ChannelMessagesStoreError> {
        let retrieved_messages = channel_messages::table
            .filter(channel_messages::tg_copy_channel_message_id.eq(tg_copy_channel_message_id))
            .first::<ChannelMessagesModel>(self.conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| {
                ChannelMessagesStoreError::Internal(InternalError::with_message(format!(
                    "Failed check for existing tg_copy_channel_message_id {}",
                    err
                )))
            })?
            .ok_or_else(|| {
                ChannelMessagesStoreError::InvalidArgument(InvalidArgumentError::new(
                    "tg_copy_channel_message_id".to_string(),
                    "A channel_messages for the given tg_copy_channel_message_id does not exist"
                        .to_string(),
                ))
            })?;

        Ok(ChannelMessages::from(retrieved_messages))
    } // end fn get_from_db_by_id
}
