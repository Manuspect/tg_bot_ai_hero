use super::UserProfileStoreOperations;

use diesel::{prelude::*, result::Error::NotFound};

use crate::modules::{
    biome::profile::store::{diesel::models::ProfileModel, Profile, UserProfileStoreError},
    error::{InternalError, InvalidArgumentError},
};
use crate::schema::user_profile;

pub trait UserProfileStoreGetProfile {
    fn get_profile(&mut self, user_id: &str) -> Result<Profile, UserProfileStoreError>;
}

#[cfg(feature = "sqlite")]
impl<'a> UserProfileStoreGetProfile
    for UserProfileStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn get_profile(&mut self, user_id: &str) -> Result<Profile, UserProfileStoreError> {
        let profile = user_profile::table
            .filter(user_profile::user_id.eq(user_id))
            .first::<ProfileModel>(self.conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| {
                UserProfileStoreError::Internal(InternalError::with_message(format!(
                    "Failed check for existing user_id {}",
                    err
                )))
            })?
            .ok_or_else(|| {
                UserProfileStoreError::InvalidArgument(InvalidArgumentError::new(
                    "user_id".to_string(),
                    "A profile for the given user_id does not exist".to_string(),
                ))
            })?;
        Ok(Profile::from(profile))
    }
}
