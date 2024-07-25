use crate::{
    modules::{
        error::{ConstraintViolationError, ConstraintViolationType, InternalError},
        prefs::store::{diesel::models::PreferencesModel, Preferences, PreferencesStoreError},
    },
    schema::preferences,
};

use super::UserPreferencesStoreOperations;

use diesel::{dsl::insert_into, result::Error::NotFound};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

pub trait PreferencesStoreAddPreference {
    fn set_value(&mut self, preference: Preferences) -> Result<(), PreferencesStoreError>;
}

#[cfg(feature = "sqlite")]
impl<'a> PreferencesStoreAddPreference
    for UserPreferencesStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn set_value(&mut self, preferences: Preferences) -> Result<(), PreferencesStoreError> {
        let duplicate_preference = preferences::table
            .filter(preferences::pref_key.eq(&preferences.key))
            .first::<PreferencesModel>(self.conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| {
                PreferencesStoreError::Internal(InternalError::with_message(format!(
                    "Failed check for existing user_id {}",
                    err
                )))
            })?;

        if duplicate_preference.is_some() {
            return Err(PreferencesStoreError::ConstraintViolation(
                ConstraintViolationError::with_violation_type(ConstraintViolationType::Unique),
            ));
        }

        insert_into(preferences::table)
            .values(PreferencesModel::from(preferences.clone()))
            .on_conflict(preferences::pref_key)
            .do_update()
            .set(PreferencesModel::from(preferences.clone()))
            .execute(self.conn)
            .map(|_| ())
            .map_err(|_| {
                PreferencesStoreError::Internal(InternalError::with_message(
                    "Failed to add credentials".to_string(),
                ))
            })?;
        Ok(())
    }
}

#[cfg(feature = "postgres")]
impl<'a> PreferencesStoreAddPreference
    for PreferencesStoreOperations<'a, diesel::pg::PgConnection>
{
    fn add_preference(&self, preference: Preference) -> Result<(), PreferencesStoreError> {
        let duplicate_preference = user_preference::table
            .filter(user_preference::user_id.eq(&preference.user_id))
            .first::<PreferenceModel>(self.conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| {
                PreferencesStoreError::Internal(InternalError::with_message(format!(
                    "Failed check for existing user_id {}",
                    err
                )))
            })?;

        if duplicate_preference.is_some() {
            return Err(PreferencesStoreError::ConstraintViolation(
                ConstraintViolationError::with_violation_type(ConstraintViolationType::Unique),
            ));
        }

        insert_into(user_preference::table)
            .values(PreferenceModel::from(preference))
            .execute(self.conn)
            .map(|_| ())
            .map_err(|_| {
                PreferencesStoreError::Internal(InternalError::with_message(
                    "Failed to add credentials".to_string(),
                ))
            })?;
        Ok(())
    }
}
