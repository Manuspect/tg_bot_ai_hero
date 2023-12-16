// This file contains a database model of channel messages model.

use chrono::{DateTime, Utc};
use diesel::{
    ExpressionMethods, Identifiable, Insertable, PgConnection, QueryDsl, Queryable, RunQueryDsl,
    Selectable, SelectableHelper,
};
use serde::{Deserialize, Serialize};

/// This structure represents a channel messages object in a database.
#[derive(
    Default, Clone, Debug, Queryable, Selectable, Identifiable, Insertable, Serialize, Deserialize,
)]
#[diesel(primary_key(message_id))]
#[diesel(table_name = crate::schema::channel_messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChannelMessages {
    pub message_id: i32,
    /// The ID of the message in the "copy tg channel".
    pub tg_copy_channel_message_id: Option<i32>,
    /// The ID of the message in the "copy vk channel".
    pub vk_copy_channel_message_id: Option<i32>,
    /// The ID of the message in the "copy inst channel".
    pub inst_copy_channel_message_id: Option<i32>,
    /// The ID of the corresponding message in the "paste channel".
    pub paste_channel_message_id: Option<i32>,
    /// The timestamp that indicates the insertion time.
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
} // end struct ChannelMessages

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::channel_messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChannelMessagesForm {
    /// The ID of the message in the "copy tg channel".
    pub tg_copy_channel_message_id: Option<i32>,
    /// The ID of the message in the "copy vk channel".
    pub vk_copy_channel_message_id: Option<i32>,
    /// The ID of the message in the "copy inst channel".
    pub inst_copy_channel_message_id: Option<i32>,
    /// The ID of the corresponding message in the "paste channel".
    pub paste_channel_message_id: Option<i32>,
}

/// Implement functionality.
impl ChannelMessages {
    /// This function inserts messages into the database.
    /// The function returns an error if the role already exists.
    /// The function also returns an error if an insertion fails.
    pub fn insert_into_db(
        channel_messages: &Self,
        conn: &mut PgConnection,
    ) -> Result<Self, diesel::result::Error> {
        let inserted_messages = diesel::insert_into(crate::schema::channel_messages::table)
            .values(channel_messages.clone())
            .get_result(conn)?;

        Ok(inserted_messages)
    } // end fn insert_into_db

    /// This function inserts ChannelMessagesForm messages into the database.
    /// The function returns an error if the role already exists.
    /// The function also returns an error if an insertion fails.
    pub fn insert_form_into_db(
        channel_messages: &ChannelMessagesForm,
        conn: &mut PgConnection,
    ) -> Result<Self, diesel::result::Error> {
        let inserted_messages = diesel::insert_into(crate::schema::channel_messages::table)
            // .values(channel_messages)
            .values((
                crate::schema::channel_messages::tg_copy_channel_message_id
                    .eq(channel_messages.tg_copy_channel_message_id),
                crate::schema::channel_messages::vk_copy_channel_message_id
                    .eq(channel_messages.vk_copy_channel_message_id),
                crate::schema::channel_messages::inst_copy_channel_message_id
                    .eq(channel_messages.inst_copy_channel_message_id),
                crate::schema::channel_messages::paste_channel_message_id
                    .eq(channel_messages.paste_channel_message_id),
            ))
            .returning(ChannelMessages::as_returning())
            .get_result(conn)?;

        Ok(inserted_messages)
    } // end fn insert_into_db

    /// This function retrieves messages by its id from the database.
    /// The function returns an error if the operation fails.
    pub fn get_from_db_by_tg_id(
        tg_copy_channel_message_id: i32,
        conn: &mut PgConnection,
    ) -> Result<Self, diesel::result::Error> {
        let retrieved_messages: Self = crate::schema::channel_messages::table
            .filter(
                crate::schema::channel_messages::dsl::tg_copy_channel_message_id
                    .eq(tg_copy_channel_message_id),
            )
            .first::<Self>(conn)?;

        Ok(retrieved_messages)
    } // end fn get_from_db_by_id

    /// This function removes all messages that are older than a particular
    /// timestamp. (This function is necessary not to overload the database
    /// with messages).s
    pub fn remove_old_messages(
        timestamp: DateTime<Utc>,
        conn: &mut PgConnection,
    ) -> Result<(), diesel::result::Error> {
        diesel::delete(crate::schema::channel_messages::table)
            .filter(crate::schema::channel_messages::dsl::updated_at.le(timestamp))
            .execute(conn)?;

        Ok(())
    } // end fn remove_old_messages
} // end impl ChannelMessages
