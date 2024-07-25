use super::UserProfileStoreOperations;

use diesel::{dsl::insert_into, result::Error::NotFound};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

use crate::modules::{
    biome::profile::store::{diesel::models::ProfileModel, Profile, UserProfileStoreError},
    error::{ConstraintViolationError, ConstraintViolationType, InternalError},
};
use crate::schema::user_profile;

pub trait UserProfileStoreAddProfile {
    fn add_profile(&mut self, profile: Profile) -> Result<(), UserProfileStoreError>;
}

#[cfg(feature = "sqlite")]
impl<'a> UserProfileStoreAddProfile
    for UserProfileStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn add_profile(&mut self, profile: Profile) -> Result<(), UserProfileStoreError> {
        let duplicate_profile = user_profile::table
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

        if duplicate_profile.is_some() {
            return Err(UserProfileStoreError::ConstraintViolation(
                ConstraintViolationError::with_violation_type(ConstraintViolationType::Unique),
            ));
        }

        insert_into(user_profile::table)
            .values(ProfileModel::from(profile))
            .execute(self.conn)
            .map(|_| ())
            .map_err(|_| {
                UserProfileStoreError::Internal(InternalError::with_message(
                    "Failed to add credentials".to_string(),
                ))
            })?;
        Ok(())
    }
}

#[cfg(feature = "postgres")]
impl<'a> UserProfileStoreAddProfile for UserProfileStoreOperations<'a, diesel::pg::PgConnection> {
    fn add_profile(&self, profile: Profile) -> Result<(), UserProfileStoreError> {
        let duplicate_profile = user_profile::table
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

        if duplicate_profile.is_some() {
            return Err(UserProfileStoreError::ConstraintViolation(
                ConstraintViolationError::with_violation_type(ConstraintViolationType::Unique),
            ));
        }

        insert_into(user_profile::table)
            .values(ProfileModel::from(profile))
            .execute(self.conn)
            .map(|_| ())
            .map_err(|_| {
                UserProfileStoreError::Internal(InternalError::with_message(
                    "Failed to add credentials".to_string(),
                ))
            })?;
        Ok(())
    }
}
