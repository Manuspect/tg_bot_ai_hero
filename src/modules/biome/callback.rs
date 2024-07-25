#[cfg(feature = "biome-profile")]
use super::profile::store::Profile;
#[cfg(feature = "biome-profile")]
use crate::modules::biome::{
    profile::store::ProfileBuilder, profile::store::UserProfileStoreError, UserProfileStore,
};
#[cfg(feature = "biome-profile")]
use crate::modules::error::InternalError;

use std::sync::Arc;

use teloxide::dptree::di::{DependencyMap, DependencySupplier};

use crate::{env_config::SharedConfig, module_mgr::Module};

pub(crate) struct BiomeService {
    config: SharedConfig,
}

impl BiomeService {
    /// Gets the user's Biome ID from the session store and saves the user profile information to
    /// the user profile store
    #[cfg(feature = "biome-profile")]
    fn save_user_profile(
        user_profile_store: Box<dyn UserProfileStore>,
        profile: &Profile,
        subject: String,
    ) -> Result<(), InternalError> {
        // let profile = ProfileBuilder::new()
        //     .with_user_id(user.user_id().into())
        //     .with_subject(profile.subject.clone())
        //     .with_name(profile.name.clone())
        //     .with_given_name(profile.given_name.clone())
        //     .with_family_name(profile.family_name.clone())
        //     .with_email(profile.email.clone())
        //     .with_picture(profile.picture.clone())
        //     .build()
        //     .map_err(|err| anyhow!(err))?;

        match user_profile_store.get_profile(profile.user_id().unwrap()) {
            Ok(_) => user_profile_store
                .update_profile(profile.clone())
                .map_err(|err| InternalError::from_source(Box::new(err))),
            Err(UserProfileStoreError::InvalidArgument(_)) => user_profile_store
                .add_profile(profile.clone())
                .map_err(|err| InternalError::from_source(Box::new(err))),
            Err(err) => Err(InternalError::from_source(Box::new(err))),
        }
    }
}

pub(crate) struct Biome;

#[async_trait]
impl Module for Biome {
    async fn register_dependency(
        &mut self,
        dep_map: &mut DependencyMap,
    ) -> Result<(), anyhow::Error> {
        let config: Arc<SharedConfig> = dep_map.get();
        let biome_service = BiomeService {
            config: config.as_ref().clone(),
        };
        dep_map.insert(biome_service);

        Ok(())
    }
}
