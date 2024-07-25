//! A memory-backed implementation of the [ChannelMessagesStore]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::modules::error::{InternalError, InvalidStateError};

use super::error::ChannelMessagesStoreError;
use super::{ChannelMessages, UserChannelMessagesStore};
use crate::modules::error::InvalidArgumentError;
#[derive(Default, Clone)]
pub struct MemoryChannelMessagesStore {
    inner: Arc<Mutex<HashMap<i32, ChannelMessages>>>,
}

impl MemoryChannelMessagesStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl UserChannelMessagesStore for MemoryChannelMessagesStore {
    fn clone_box(&self) -> Box<dyn UserChannelMessagesStore> {
        Box::new(self.clone())
    }

    fn add_channel_message(
        &self,
        channel_message: ChannelMessages,
    ) -> Result<(), ChannelMessagesStoreError> {
        let mut inner = self.inner.lock().map_err(|err| {
            ChannelMessagesStoreError::Internal(InternalError::with_message(err.to_string()))
        })?;
        let id_size = inner.keys().len() as i32;
        if inner.get(&id_size).is_some() {
            return Err(ChannelMessagesStoreError::InvalidArgument(
                InvalidArgumentError::new(
                    "key".to_string(),
                    "A channel_messages for the given key already exists".to_string(),
                ),
            ));
        }
        inner.insert(id_size.clone(), channel_message);

        Ok(())
    }

    fn delete_channel_message(&self, message_id: &i32) -> Result<(), ChannelMessagesStoreError> {
        let mut inner = self.inner.lock().map_err(|_| {
            ChannelMessagesStoreError::Internal(InternalError::with_message(
                "Cannot access user profile store: mutex lock poisoned".to_string(),
            ))
        })?;
        if inner.remove(message_id).is_some() {
            Ok(())
        } else {
            Err(ChannelMessagesStoreError::InvalidState(
                InvalidStateError::with_message(
                    "A channel_message with the given message_id does not exist".to_string(),
                ),
            ))
        }
    }

    fn get_channel_message(
        &self,
        message_id: &i32,
    ) -> Result<ChannelMessages, ChannelMessagesStoreError> {
        let inner = self.inner.lock().map_err(|err| {
            ChannelMessagesStoreError::Internal(InternalError::with_message(err.to_string()))
        })?;

        match inner.get(message_id).cloned() {
            Some(channel_message) => Ok(channel_message),
            None => Err(ChannelMessagesStoreError::InvalidArgument(
                InvalidArgumentError::new(
                    "message_id".to_string(),
                    "A channel_messages for the given key does not exist".to_string(),
                ),
            )),
        }
    }

    fn get_from_db_by_tg_id(
        &mut self,
        tg_copy_channel_message_id: &i32,
    ) -> Result<ChannelMessages, ChannelMessagesStoreError> {
        let inner = self.inner.lock().map_err(|err| {
            ChannelMessagesStoreError::Internal(InternalError::with_message(err.to_string()))
        })?;

        match inner.get(tg_copy_channel_message_id).cloned() {
            Some(channel_message) => Ok(channel_message),
            None => Err(ChannelMessagesStoreError::InvalidArgument(
                InvalidArgumentError::new(
                    "tg_copy_channel_message_id".to_string(),
                    "A channel_messages for the given tg_copy_channel_message_id does not exist"
                        .to_string(),
                ),
            )),
        }
    }

    fn remove_old_messages(
        &mut self,
        timestamp: chrono::NaiveDateTime,
    ) -> Result<(), ChannelMessagesStoreError> {
        let mut inner = self.inner.lock().map_err(|err| {
            ChannelMessagesStoreError::Internal(InternalError::with_message(err.to_string()))
        })?;
        let mut keys_to_remove = Vec::new();
        for (key, value) in inner.iter() {
            if value.created_at < timestamp {
                keys_to_remove.push(key.clone());
            }
        }
        for key in keys_to_remove {
            inner.remove(&key);
        }
        Ok(())
    }
}
