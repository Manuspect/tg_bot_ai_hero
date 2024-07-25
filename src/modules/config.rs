use anyhow::Error;
use std::time::SystemTime;
use teloxide::prelude::*;

use crate::{env_config::SharedConfig, module_mgr::Module};

use super::error::InternalError;

pub(crate) struct Config {
    config: Option<SharedConfig>,
}

impl Config {
    pub(crate) fn new(config: SharedConfig) -> Self {
        Self {
            config: Some(config),
        }
    }
}

#[async_trait]
impl Module for Config {
    async fn register_dependency(&mut self, dep_map: &mut DependencyMap) -> Result<(), Error> {
        dep_map.insert(self.config.take().unwrap());
        Ok(())
    }
}

pub fn get_current_time() -> chrono::NaiveDateTime {
    let time = SystemTime::now();
    let duration = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|err| InternalError::with_message(err.to_string()))
        .unwrap();
    let seconds = i64::try_from(duration.as_secs())
        .map_err(|err| InternalError::with_message(err.to_string()))
        .unwrap();
    chrono::NaiveDateTime::from_timestamp_opt(seconds, duration.subsec_nanos()).unwrap()
}

pub fn calculate_time(age: std::time::Duration) -> chrono::NaiveDateTime {
    let time = SystemTime::now();
    let current_duration = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|err| InternalError::with_message(err.to_string()))
        .unwrap();
    let duration = current_duration - age;
    let seconds = i64::try_from(duration.as_secs())
        .map_err(|err| InternalError::with_message(err.to_string()))
        .unwrap();
    chrono::NaiveDateTime::from_timestamp_opt(seconds, duration.subsec_nanos()).unwrap()
}
