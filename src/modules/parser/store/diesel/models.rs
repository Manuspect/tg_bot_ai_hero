use serde::Deserialize;

use crate::{modules::parser::store::ChannelMessages, schema::channel_messages};

#[derive(
    Insertable,
    Selectable,
    Queryable,
    QueryableByName,
    Identifiable,
    PartialEq,
    Eq,
    Debug,
    Deserialize,
    AsChangeset,
)]
#[table_name = "channel_messages"]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[primary_key(message_id)]
pub struct ChannelMessagesModel {
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
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(
    Insertable,
    Selectable,
    Queryable,
    QueryableByName,
    PartialEq,
    Eq,
    Debug,
    Deserialize,
    AsChangeset,
)]
#[table_name = "channel_messages"]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewChannelMessagesModel {
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

impl From<ChannelMessages> for NewChannelMessagesModel {
    fn from(channel_messages: ChannelMessages) -> Self {
        NewChannelMessagesModel {
            tg_copy_channel_message_id: channel_messages.tg_copy_channel_message_id,
            vk_copy_channel_message_id: channel_messages.vk_copy_channel_message_id,
            inst_copy_channel_message_id: channel_messages.inst_copy_channel_message_id,
            paste_channel_message_id: channel_messages.paste_channel_message_id,
            created_at: channel_messages.created_at,
            updated_at: channel_messages.updated_at,
        }
    }
}
