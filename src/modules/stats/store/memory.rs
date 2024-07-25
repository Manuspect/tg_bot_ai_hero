//! A memory-backed implementation of the [TokenUsageStore]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::modules::error::InternalError;

use super::error::TokenUsageStoreError;
use super::{TokenUsage, UserTokenUsageStore};

#[derive(Default, Clone)]
pub struct MemoryTokenUsageStore {
    inner: Arc<Mutex<HashMap<(String, chrono::NaiveDateTime), TokenUsage>>>,
}

impl MemoryTokenUsageStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl UserTokenUsageStore for MemoryTokenUsageStore {
    fn add_token_usage(&self, token_usage: TokenUsage) -> Result<(), TokenUsageStoreError> {
        let mut inner = self.inner.lock().map_err(|_| {
            TokenUsageStoreError::Internal(InternalError::with_message(
                "Cannot token usage token_usage store: mutex lock poisoned".to_string(),
            ))
        })?;

        inner.insert(
            (token_usage.user_id.clone(), token_usage.time.clone()),
            token_usage,
        );
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn UserTokenUsageStore> {
        Box::new(self.clone())
    }

    fn query_usage_of_user(&self, user_id: &str) -> Result<i64, TokenUsageStoreError> {
        let inner = self.inner.lock().map_err(|_| {
            TokenUsageStoreError::Internal(InternalError::with_message(
                "Cannot token usage token_usage store: mutex lock poisoned".to_string(),
            ))
        })?;
        Ok(inner
            .iter()
            .filter(|x| x.0 .0 == user_id)
            .map(|x| x.1.tokens)
            .sum())
    }

    fn query_total_usage(&self) -> Result<i64, TokenUsageStoreError> {
        let inner = self.inner.lock().map_err(|_| {
            TokenUsageStoreError::Internal(InternalError::with_message(
                "Cannot access token usage store: mutex lock poisoned".to_string(),
            ))
        })?;
        Ok(inner.iter().map(|x| x.1.tokens).sum())
    }
}
