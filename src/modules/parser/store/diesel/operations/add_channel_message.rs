use crate::{
    modules::{
        error::{ConstraintViolationError, ConstraintViolationType, InternalError},
        parser::store::{
            diesel::models::{ChannelMessagesModel, NewChannelMessagesModel},
            ChannelMessages, ChannelMessagesStoreError,
        },
    },
    schema::channel_messages,
};

use super::UserChannelMessagesStoreOperations;

use diesel::{dsl::insert_into, result::Error::NotFound};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

pub trait ChannelMessagesStoreAddMessages {
    fn add_channel_message(
        &mut self,
        channel_message: ChannelMessages,
    ) -> Result<(), ChannelMessagesStoreError>;
}

#[cfg(feature = "sqlite")]
impl<'a> ChannelMessagesStoreAddMessages
    for UserChannelMessagesStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn add_channel_message(
        &mut self,
        channel_messages: ChannelMessages,
    ) -> Result<(), ChannelMessagesStoreError> {
        let duplicate_channel_message = channel_messages::table
            .filter(
                channel_messages::tg_copy_channel_message_id
                    .eq(&channel_messages.tg_copy_channel_message_id),
            )
            .first::<ChannelMessagesModel>(self.conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| {
                ChannelMessagesStoreError::Internal(InternalError::with_message(format!(
                    "Failed check for existing user_id {}",
                    err
                )))
            })?;

        if duplicate_channel_message.is_some() {
            return Err(ChannelMessagesStoreError::ConstraintViolation(
                ConstraintViolationError::with_violation_type(ConstraintViolationType::Unique),
            ));
        }

        insert_into(channel_messages::table)
            .values(NewChannelMessagesModel::from(channel_messages.clone()))
            .on_conflict(channel_messages::message_id)
            .do_update()
            .set(NewChannelMessagesModel::from(channel_messages.clone()))
            .execute(self.conn)
            .map(|_| ())
            .map_err(|_| {
                ChannelMessagesStoreError::Internal(InternalError::with_message(
                    "Failed to add credentials".to_string(),
                ))
            })?;
        Ok(())
    }
}
