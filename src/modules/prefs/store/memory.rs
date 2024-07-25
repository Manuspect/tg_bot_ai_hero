//! A memory-backed implementation of the [PreferencesStore]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::modules::error::InternalError;

use super::error::PreferencesStoreError;
use super::{Preferences, UserPreferencesStore};
use crate::modules::error::InvalidArgumentError;
#[derive(Default, Clone)]
pub struct MemoryPreferencesStore {
    inner: Arc<Mutex<HashMap<String, Preferences>>>,
}

impl MemoryPreferencesStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl UserPreferencesStore for MemoryPreferencesStore {
    fn clone_box(&self) -> Box<dyn UserPreferencesStore> {
        Box::new(self.clone())
    }

    fn set_value(&self, prefs: Preferences) -> Result<(), PreferencesStoreError> {
        let mut inner = self.inner.lock().map_err(|err| {
            PreferencesStoreError::Internal(InternalError::with_message(err.to_string()))
        })?;

        inner.insert(prefs.key.clone(), prefs);

        Ok(())
    }

    fn get_value(&self, key: &str) -> Result<Preferences, PreferencesStoreError> {
        let inner = self.inner.lock().map_err(|err| {
            PreferencesStoreError::Internal(InternalError::with_message(err.to_string()))
        })?;

        match inner.get(key).cloned() {
            Some(prefs) => Ok(prefs),
            None => Err(PreferencesStoreError::InvalidArgument(
                InvalidArgumentError::new(
                    "key".to_string(),
                    "A preferences for the given key does not exist".to_string(),
                ),
            )),
        }
    }
}
