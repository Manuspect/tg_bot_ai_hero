use anyhow::Error;
use build_database::build_database::Migrate;
use serde::{Deserialize, Serialize};

use crate::{
    database::{self, DatabaseManager},
    env_config::SharedConfig,
    modules::{config::get_current_time, error::ServiceStartError, prefs::PreferencesManager},
};

use super::store::MembersBuilder;

const PUBLIC_USABLE_PREF_KEY: &str = "PublicUsable";

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct PublicUsableValue(bool);

impl Default for PublicUsableValue {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Clone)]
pub(crate) struct MemberManager {
    db_mgr: DatabaseManager,
    pref_mgr: PreferencesManager,
    config: SharedConfig,
}

impl MemberManager {
    pub async fn new(
        db_mgr: DatabaseManager,
        pref_mgr: PreferencesManager,
        config: SharedConfig,
    ) -> Result<Self, Error> {
        // Initialize the database table before returning.
        match Migrate::run(None) {
            Ok(_) => Ok(Self {
                db_mgr,
                pref_mgr,
                config,
            }),
            Err(e) => Err(anyhow!("Failed to initialize database table {:?}", e)),
        }
    }

    pub async fn add_member(&self, username: String) -> Result<bool, Error> {
        let result = self
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

                let member = MembersBuilder::new()
                    .with_username(username.to_string())
                    .with_disabled(0)
                    .with_created_at(get_current_time())
                    .build()
                    .unwrap();

                match store_factory.get_member_store().add_member(member) {
                    Ok(_) => true,
                    Err(e) => {
                        log::error!("{}", e);
                        false
                    }
                }
            })
            .await?;

        Ok(result)
    }

    pub async fn delete_member(&self, username: String) -> Result<bool, Error> {
        let result = self
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

                match store_factory.get_member_store().delete_member(&username) {
                    Ok(_) => return true,
                    Err(e) => log::error!("{}", e),
                }

                false
            })
            .await?;

        Ok(result)
    }

    pub async fn is_member_allowed(&self, username: String) -> Result<bool, Error> {
        let public_usable: PublicUsableValue =
            self.pref_mgr.get_value(PUBLIC_USABLE_PREF_KEY).await?;
        if public_usable.0 {
            return Ok(true);
        }

        if self.config.admin_usernames.contains(&username) {
            return Ok(true);
        }

        let result = self
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
                match store_factory.get_member_store().get_member(&username) {
                    Ok(member) => {
                        if member.disabled().clone() == 1 {
                            return false;
                        }
                        true
                    }
                    Err(e) => {
                        log::error!("{}", e);
                        false
                    }
                }
            })
            .await?;

        Ok(result)
    }

    pub async fn set_public_usable(&self, public_usable: bool) -> Result<(), Error> {
        self.pref_mgr
            .set_value(PUBLIC_USABLE_PREF_KEY, &PublicUsableValue(public_usable))
            .await?;
        Ok(())
    }
}
