//! Provides [`ChannelMessagesStore`](crate::biome::profile::store::ChannelMessagesStore) operations
//! implemented for a diesel backend

pub(super) mod add_channel_message;
pub(super) mod delete_channel_message;
pub(super) mod get_channel_message;
pub(super) struct UserChannelMessagesStoreOperations<'a, C> {
    conn: &'a mut C,
}

impl<'a, C> UserChannelMessagesStoreOperations<'a, C>
where
    C: diesel::Connection,
{
    pub fn new(conn: &'a mut C) -> Self {
        UserChannelMessagesStoreOperations { conn }
    }
}
