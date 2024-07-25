use super::UserProfileStoreOperations;

use diesel::{dsl::delete, prelude::*, result::Error::NotFound};

use crate::schema::user_profile;

use crate::modules::{
    biome::profile::store::{diesel::models::ProfileModel, UserProfileStoreError},
    error::{InternalError, InvalidArgumentError},
};

pub trait UserProfileStoreRemoveProfile {
    fn remove_profile(&mut self, user_id: &str) -> Result<(), UserProfileStoreError>;
}

#[cfg(feature = "sqlite")]
impl<'a> UserProfileStoreRemoveProfile
    for UserProfileStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn remove_profile(&mut self, user_id: &str) -> Result<(), UserProfileStoreError> {
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
            })?;
        if profile.is_none() {
            return Err(UserProfileStoreError::InvalidArgument(
                InvalidArgumentError::new(
                    "user_id".to_string(),
                    "A profile for the given user_id does not exist".to_string(),
                ),
            ));
        }

        delete(user_profile::table.filter(user_profile::user_id.eq(user_id)))
            .execute(self.conn)
            .map(|_| ())
            .map_err(|err| {
                UserProfileStoreError::Internal(InternalError::with_message(format!(
                    "Failed check for existing user_id {}",
                    err
                )))
            })?;
        Ok(())
    }
}
