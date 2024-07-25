use super::UserProfileStoreOperations;

use diesel::{dsl::update, prelude::*, result::Error::NotFound};

use crate::schema::user_profile;

use crate::modules::{
    biome::profile::store::{diesel::models::ProfileModel, Profile, UserProfileStoreError},
    error::{InternalError, InvalidArgumentError},
};

pub trait UserProfileStoreUpdateProfile {
    fn update_profile(&mut self, profile: Profile) -> Result<(), UserProfileStoreError>;
}

#[cfg(feature = "sqlite")]
impl<'a> UserProfileStoreUpdateProfile
    for UserProfileStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn update_profile(&mut self, profile: Profile) -> Result<(), UserProfileStoreError> {
        let profile_exists = user_profile::table
            .filter(user_profile::user_id.eq(&profile.user_id))
            .first::<ProfileModel>(self.conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| {
                UserProfileStoreError::Internal(InternalError::with_message(format!(
                    "Failed check for existing user_id {}",
                    err
                )))
            })?;
        if profile_exists.is_none() {
            return Err(UserProfileStoreError::InvalidArgument(
                InvalidArgumentError::new(
                    "user_id".to_string(),
                    "A profile for the given user_id does not exist".to_string(),
                ),
            ));
        }
        update(user_profile::table.filter(user_profile::user_id.eq(&profile.user_id)))
            .set((
                user_profile::user_id.eq(&profile.user_id),
                user_profile::name.eq(&profile.name),
                user_profile::given_name.eq(&profile.given_name),
                user_profile::family_name.eq(&profile.family_name),
                user_profile::email.eq(&profile.email),
                user_profile::picture.eq(&profile.picture),
            ))
            .execute(self.conn)
            .map(|_| ())
            .map_err(|err| {
                UserProfileStoreError::Internal(InternalError::with_message(format!(
                    "Failed to update profile {}",
                    err
                )))
            })?;
        Ok(())
    }
}
