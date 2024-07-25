//! A memory-backed implementation of the [MembersStore]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::modules::error::{InternalError, InvalidStateError};

use super::error::MembersStoreError;
use super::{Members, UserMembersStore};
use crate::modules::error::InvalidArgumentError;
#[derive(Default, Clone)]
pub struct MemoryMembersStore {
    inner: Arc<Mutex<HashMap<String, Members>>>,
}

impl MemoryMembersStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl UserMembersStore for MemoryMembersStore {
    fn clone_box(&self) -> Box<dyn UserMembersStore> {
        Box::new(self.clone())
    }

    fn add_member(&self, member: Members) -> Result<(), MembersStoreError> {
        let mut inner = self.inner.lock().map_err(|err| {
            MembersStoreError::Internal(InternalError::with_message(err.to_string()))
        })?;

        if inner.get(&member.username).is_some() {
            return Err(MembersStoreError::InvalidArgument(
                InvalidArgumentError::new(
                    "key".to_string(),
                    "A members for the given key already exists".to_string(),
                ),
            ));
        }
        inner.insert(member.username.clone(), member);

        Ok(())
    }

    fn delete_member(&self, username: &str) -> Result<(), MembersStoreError> {
        let mut inner = self.inner.lock().map_err(|_| {
            MembersStoreError::Internal(InternalError::with_message(
                "Cannot access user profile store: mutex lock poisoned".to_string(),
            ))
        })?;
        if inner.remove(username).is_some() {
            Ok(())
        } else {
            Err(MembersStoreError::InvalidState(
                InvalidStateError::with_message(
                    "A member with the given username does not exist".to_string(),
                ),
            ))
        }
    }

    fn get_member(&self, username: &str) -> Result<Members, MembersStoreError> {
        let inner = self.inner.lock().map_err(|err| {
            MembersStoreError::Internal(InternalError::with_message(err.to_string()))
        })?;

        match inner.get(username).cloned() {
            Some(member) => Ok(member),
            None => Err(MembersStoreError::InvalidArgument(
                InvalidArgumentError::new(
                    "username".to_string(),
                    "A members for the given key does not exist".to_string(),
                ),
            )),
        }
    }
}
