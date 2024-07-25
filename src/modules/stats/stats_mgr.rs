use anyhow::Error;
use build_database::build_database::Migrate;
// use rusqlite::{Connection as SqliteConnection, OptionalExtension};

use crate::{
    database::{self, DatabaseManager},
    modules::{
        config::get_current_time,
        error::{InternalError, ServiceStartError},
    },
};

use super::store::TokenUsageBuilder;

#[derive(Clone)]
pub(crate) struct StatsManager {
    db_mgr: DatabaseManager,
}

impl StatsManager {
    pub async fn with_db_manager(db_mgr: DatabaseManager) -> Result<Self, Error> {
        // Initialize the database table before returning.
        match Migrate::run(None) {
            Ok(_) => Ok(Self { db_mgr }),
            Err(e) => Err(anyhow!("Failed to initialize database table {:?}", e)),
        }
    }

    pub async fn add_usage(&self, user_id: String, tokens: i64) -> Result<(), InternalError> {
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

                let user_id = &user_id;

                let token_usage = TokenUsageBuilder::new()
                    .with_user_id(user_id.to_string())
                    .with_time(get_current_time())
                    .with_tokens(tokens)
                    .build()
                    .unwrap();

                match store_factory.get_stats_store().add_token_usage(token_usage) {
                    Ok(_) => {}
                    Err(e) => log::error!("{}", e),
                }
            })
            .await
            .unwrap();

        Ok(())
    }

    pub async fn query_usage(&self, user_id: Option<String>) -> Result<i64, Error> {
        let usage = self
            .db_mgr
            .query(|conn| {
                let store_factory = database::create_store_factory(conn)
                    .map_err(|err| {
                        ServiceStartError::StorageError(format!(
                            "Failed to initialize store factory: {}",
                            err
                        ))
                    })
                    .unwrap();

                let usage = if let Some(user_id) = user_id {
                    store_factory
                        .get_stats_store()
                        .query_usage_of_user(&user_id)
                } else {
                    store_factory.get_stats_store().query_total_usage()
                };

                match usage {
                    Ok(usage) => usage,
                    Err(err) => {
                        log::error!("Failed to query usage: {}", err);
                        0
                    }
                }
            })
            .await?;

        Ok(usage)
    }
}
