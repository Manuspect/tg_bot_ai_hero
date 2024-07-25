use super::UserProfileStoreOperations;

use diesel::prelude::*;

use crate::schema::user_profile;

use crate::modules::{
    biome::profile::store::{diesel::models::ProfileModel, Profile, UserProfileStoreError},
    error::{InternalError, InvalidArgumentError},
};

pub trait UserProfileStorelistProfiles {
    fn list_profiles(&mut self) -> Result<Option<Vec<Profile>>, UserProfileStoreError>;
}

#[cfg(feature = "sqlite")]
impl<'a> UserProfileStorelistProfiles
    for UserProfileStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn list_profiles(&mut self) -> Result<Option<Vec<Profile>>, UserProfileStoreError> {
        let profiles = user_profile::table
            .select(user_profile::all_columns)
            .load::<ProfileModel>(self.conn)
            .map(Some)
            .map_err(|err| {
                UserProfileStoreError::Internal(InternalError::with_message(format!(
                    "Failed to get profiles {}",
                    err
                )))
            })?
            .ok_or_else(|| {
                UserProfileStoreError::InvalidArgument(InvalidArgumentError::new(
                    "user_id".to_string(),
                    "A profile for the given user_id does not exist".to_string(),
                ))
            })?
            .into_iter()
            .map(Profile::from)
            .collect();
        Ok(Some(profiles))
    }
}
