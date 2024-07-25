//! Database-backed implementation of the [ChannelMessagesStore], powered by [diesel].

pub mod models;
mod operations;

use std::sync::{Arc, RwLock};

use diesel::r2d2::{ConnectionManager, Pool};

use crate::store::pool::ConnectionPool;

use super::{ChannelMessages, ChannelMessagesStoreError, UserChannelMessagesStore};

use models::ChannelMessagesModel;

use operations::{
    add_channel_message::ChannelMessagesStoreAddMessages,
    delete_channel_message::ChannelMessagesStoreDeleteMessages,
    get_channel_message::ChannelMessagesStoreGetMessages, UserChannelMessagesStoreOperations,
};

/// Manages creating, updating, and fetching channel_messages from the database
pub struct DieselChannelMessagesStore<C: diesel::r2d2::R2D2Connection + 'static> {
    connection_pool: ConnectionPool<C>,
}

impl<C: diesel::r2d2::R2D2Connection> DieselChannelMessagesStore<C> {
    /// Creates a new DieselChannelMessagesStore
    ///
    /// # Arguments
    ///
    ///  * `connection_pool`: connection pool to the database
    pub fn new(connection_pool: Pool<ConnectionManager<C>>) -> Self {
        DieselChannelMessagesStore {
            connection_pool: connection_pool.into(),
        }
    }

    /// Create a new `DieselChannelMessagesStore` with write exclusivity enabled.
    ///
    /// Write exclusivity is enforced by providing a connection pool that is wrapped in a
    /// [`RwLock`]. This ensures that there may be only one writer, but many readers.
    ///
    /// # Arguments
    ///
    ///  * `connection_pool`: read-write lock-guarded connection pool for the database
    pub fn new_with_write_exclusivity(
        connection_pool: Arc<RwLock<Pool<ConnectionManager<C>>>>,
    ) -> Self {
        Self {
            connection_pool: connection_pool.into(),
        }
    }
}

#[cfg(feature = "postgres")]
impl ChannelMessagesStore for DieselChannelMessagesStore<diesel::pg::PgConnection> {
    fn add_profile(&self, profile: Messages) -> Result<(), ChannelMessagesStoreError> {
        self.connection_pool.execute_write(|connection| {
            ChannelMessagesStoreOperations::new(connection).add_profile(profile)
        })
    }

    fn clone_box(&self) -> Box<dyn ChannelMessagesStore> {
        Box::new(Self {
            connection_pool: self.connection_pool.clone(),
        })
    }
}

#[cfg(feature = "sqlite")]
impl UserChannelMessagesStore for DieselChannelMessagesStore<diesel::sqlite::SqliteConnection> {
    fn clone_box(&self) -> Box<dyn UserChannelMessagesStore> {
        Box::new(Self {
            connection_pool: self.connection_pool.clone(),
        })
    }

    fn add_channel_message(
        &self,
        channel_message: ChannelMessages,
    ) -> Result<(), ChannelMessagesStoreError> {
        self.connection_pool
            .execute_write(|connection: &mut diesel::prelude::SqliteConnection| {
                UserChannelMessagesStoreOperations::new(connection)
                    .add_channel_message(channel_message)
            })
    }

    fn delete_channel_message(&self, message_id: &i32) -> Result<(), ChannelMessagesStoreError> {
        self.connection_pool.execute_write(|connection| {
            UserChannelMessagesStoreOperations::new(connection).delete_channel_message(message_id)
        })
    }

    fn get_channel_message(
        &self,
        message_id: &i32,
    ) -> Result<ChannelMessages, ChannelMessagesStoreError> {
        self.connection_pool.execute_read(|connection| {
            UserChannelMessagesStoreOperations::new(connection).get_channel_message(message_id)
        })
    }

    fn get_from_db_by_tg_id(
        &mut self,
        tg_copy_channel_message_id: &i32,
    ) -> Result<ChannelMessages, ChannelMessagesStoreError> {
        self.connection_pool.execute_read(|connection| {
            UserChannelMessagesStoreOperations::new(connection)
                .get_from_db_by_tg_id(tg_copy_channel_message_id)
        })
    }

    fn remove_old_messages(
        &mut self,
        timestamp: chrono::NaiveDateTime,
    ) -> Result<(), ChannelMessagesStoreError> {
        self.connection_pool.execute_write(|connection| {
            UserChannelMessagesStoreOperations::new(connection).remove_old_messages(timestamp)
        })
    }
}

impl From<ChannelMessagesModel> for ChannelMessages {
    fn from(user_channel_messages: ChannelMessagesModel) -> Self {
        Self {
            tg_copy_channel_message_id: user_channel_messages.tg_copy_channel_message_id,
            paste_channel_message_id: user_channel_messages.paste_channel_message_id,
            vk_copy_channel_message_id: user_channel_messages.vk_copy_channel_message_id,
            inst_copy_channel_message_id: user_channel_messages.inst_copy_channel_message_id,
            created_at: user_channel_messages.created_at,
            updated_at: user_channel_messages.updated_at,
        }
    }
}
