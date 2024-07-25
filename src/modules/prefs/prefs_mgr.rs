use std::fmt::Debug;

use anyhow::Error;
use build_database::build_database::Migrate;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    database::{self, DatabaseManager},
    modules::error::ServiceStartError,
};

use super::store::PreferencesBuilder;

#[derive(Clone)]
pub(crate) struct PreferencesManager {
    db_mgr: DatabaseManager,
}

impl PreferencesManager {
    pub async fn with_db_manager(db_mgr: DatabaseManager) -> Result<Self, Error> {
        // Initialize the database table before returning.
        match Migrate::run(None) {
            Ok(_) => Ok(Self { db_mgr }),
            Err(e) => Err(anyhow!("Failed to initialize database table {:?}", e)),
        }
    }

    pub async fn set_value<V>(&self, key: &str, value: &V) -> Result<(), Error>
    where
        V: Serialize,
    {
        let key = key.to_owned();
        let serialized_value = serde_json::to_string(value)?;

        self.db_mgr
            .enqueue_work(move |conn| {
                let store_factory = database::create_store_factory(conn)
                    .map_err(|err| {
                        ServiceStartError::StorageError(format!(
                            "Failed to initialize store factory: {}",
                            err
                        ))
                    })
                    .unwrap();
                let pref = PreferencesBuilder::new()
                    .with_key(key)
                    .with_value(serialized_value)
                    .build()
                    .unwrap();

                let prefs = store_factory.get_prefs_store().set_value(pref);
                match prefs {
                    Ok(_) => {}
                    Err(err) => {
                        log::error!("Failed to query usage: {}", err);
                    }
                }
            })
            .await?;

        Ok(())
    }

    pub async fn get_value<V>(&self, key: &str) -> Result<V, Error>
    where
        V: DeserializeOwned + Default + Send + Debug + 'static,
    {
        let key = key.to_owned();
        let value = self
            .db_mgr
            .query::<_, Result<V, Error>>(move |conn| {
                let store_factory = database::create_store_factory(conn)
                    .map_err(|err| {
                        ServiceStartError::StorageError(format!(
                            "Failed to initialize store factory: {}",
                            err
                        ))
                    })
                    .unwrap();
                let prefs = store_factory.get_prefs_store().get_value(&key);
                match prefs {
                    Ok(prefs) => match prefs.value() {
                        Some(value) => {
                            let value = serde_json::from_str(value)?;
                            Ok(value)
                        }
                        None => Ok(V::default()),
                    },
                    Err(err) => {
                        log::error!("Failed to query valur: {}", err);
                        Ok(V::default())
                    }
                }
            })
            .await
            .and_then(|res| res.map_err(|err| anyhow!(err)))?;

        Ok(value)
    }
}
