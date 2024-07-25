use std::collections::HashMap;

use anyhow::Error;
use build_database::build_database::Migrate;
// use rusqlite::{Connection as SqliteConnection, OptionalExtension};

use crate::{
    database::{self, DatabaseManager},
    env_config::{self, SharedConfig},
    modules::{
        config::get_current_time,
        error::{InternalError, ServiceStartError},
    },
};

use super::store::{ChannelMessages};

#[derive(Clone)]
pub(crate) struct ParserManager {
    db_mgr: DatabaseManager,
    pub config: SharedConfig,
    pub tg_client: grammers_client::Client,
    pub tasks_to_join: std::sync::Arc<tokio::sync::Mutex<HashMap<String, Vec<String>>>>,
    pub tx_token: std::sync::Arc<tokio::sync::Mutex<Option<tokio::sync::mpsc::Sender<String>>>>,
}

impl ParserManager {
    pub(crate) async fn with_db_manager(
        db_mgr: DatabaseManager,
        config: SharedConfig,
        tg_client: grammers_client::Client,
    ) -> Result<Self, Error> {
        // Initialize the database table before returning.
        match Migrate::run(None) {
            Ok(_) => {
                let mut tasks_to_join = HashMap::new();
                tasks_to_join.insert("actor".to_string(), vec![]);

                Ok(Self {
                    db_mgr,
                    config,
                    tg_client,
                    tasks_to_join: std::sync::Arc::new(tokio::sync::Mutex::new(tasks_to_join)),
                    tx_token: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
                })
            }
            Err(e) => Err(anyhow!("Failed to initialize database table {:?}", e)),
        }
    }

    pub async fn add_channel_message(
        &self,
        channel_messages: ChannelMessages
    ) -> Result<(), InternalError> {
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
                match store_factory
                    .get_channel_message_store()
                    .add_channel_message(channel_messages)
                {
                    Ok(_) => {}
                    Err(e) => log::error!("{}", e),
                }
            })
            .await
            .unwrap();

        Ok(())
    }


    pub async fn remove_old_messages(&self, timestamp: chrono::NaiveDateTime) -> Result<(), Error> {
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
                match store_factory
                    .get_channel_message_store()
                    .remove_old_messages(timestamp)
                {
                    Ok(_) => {}
                    Err(e) => log::error!("{}", e),
                }
            })
            .await
            .unwrap();
        Ok(())
    }

    pub async fn get_channel_message(&self, message_id: i32) -> Result<ChannelMessages, Error> {
        let channel_message = self
            .db_mgr
            .query(move |conn| {
                let store_factory = database::create_store_factory(conn)
                    .map_err(|err| {
                        ServiceStartError::StorageError(format!(
                            "Failed to initialize store factory: {}",
                            err
                        ))
                    })
                    .unwrap();

                store_factory
                    .get_channel_message_store()
                    .get_channel_message(&message_id)
                    .unwrap()
            })
            .await?;

        Ok(channel_message)
    }

    pub async fn get_paste_channel_message_id(
        &self,
        message_id: i32,
    ) -> Result<Option<Option<i32>>, Error> {
        let paste_channel_message_id = self
            .db_mgr
            .query(move |conn| {
                let store_factory = database::create_store_factory(conn)
                    .map_err(|err| {
                        ServiceStartError::StorageError(format!(
                            "Failed to initialize store factory: {}",
                            err
                        ))
                    })
                    .unwrap();

                let temp = match store_factory
                    .get_channel_message_store()
                    .get_channel_message(&message_id) {
                    Ok(res) => Ok(Some(res.paste_channel_message_id)),
                    Err(err) => {
                        match err {
                            crate::modules::parser::store::ChannelMessagesStoreError::InvalidArgument(_) => Ok(None),
                            err => Err(err), 
                        }
                    }
                }.unwrap();
                return temp;
                
            })
            .await?;

        Ok(paste_channel_message_id)
    }

}
