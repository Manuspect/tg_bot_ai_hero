use crate::{
    modules::{
        error::{InternalError, InvalidArgumentError},
        parser::store::{diesel::models::ChannelMessagesModel, ChannelMessagesStoreError},
    },
    schema::channel_messages,
};

use super::UserChannelMessagesStoreOperations;

use diesel::{delete, result::Error::NotFound};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

pub trait ChannelMessagesStoreDeleteMessages {
    fn delete_channel_message(
        &mut self,
        channel_message: &i32,
    ) -> Result<(), ChannelMessagesStoreError>;
    fn remove_old_messages(
        &mut self,
        timestamp: chrono::NaiveDateTime,
    ) -> Result<(), ChannelMessagesStoreError>;
}

#[cfg(feature = "sqlite")]
impl<'a> ChannelMessagesStoreDeleteMessages
    for UserChannelMessagesStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn delete_channel_message(
        &mut self,
        message_id: &i32,
    ) -> Result<(), ChannelMessagesStoreError> {
        let profile = channel_messages::table
            .filter(channel_messages::message_id.eq(message_id))
            .first::<ChannelMessagesModel>(self.conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| {
                ChannelMessagesStoreError::Internal(InternalError::with_message(format!(
                    "Failed check for existing message_id {}",
                    err
                )))
            })?;
        if profile.is_none() {
            return Err(ChannelMessagesStoreError::InvalidArgument(
                InvalidArgumentError::new(
                    "message_id".to_string(),
                    "A channel_message for the given message_id does not exist".to_string(),
                ),
            ));
        }

        delete(channel_messages::table.filter(channel_messages::message_id.eq(message_id)))
            .execute(self.conn)
            .map(|_| ())
            .map_err(|err| {
                ChannelMessagesStoreError::Internal(InternalError::with_message(format!(
                    "Failed check for existing message_id {}",
                    err
                )))
            })?;
        Ok(())
    }

    /// This function removes all messages that are older than a particular
    /// timestamp. (This function is necessary not to overload the database
    /// with messages).s
    fn remove_old_messages(
        &mut self,
        timestamp: chrono::NaiveDateTime,
    ) -> Result<(), ChannelMessagesStoreError> {
        diesel::delete(channel_messages::table)
            .filter(channel_messages::dsl::updated_at.le(timestamp))
            .execute(self.conn)
            .map(|_| ())
            .map_err(|err| {
                ChannelMessagesStoreError::Internal(InternalError::with_message(format!(
                    "Failed removes all messages that are older than a particular timestamp {}",
                    err
                )))
            })?;

        Ok(())
    } // end fn remove_old_messages
}
