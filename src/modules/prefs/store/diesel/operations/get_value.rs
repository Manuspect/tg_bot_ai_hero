use crate::modules::prefs::store::{
    diesel::models::PreferencesModel, Preferences, PreferencesStoreError,
};

use super::UserPreferencesStoreOperations;

use diesel::RunQueryDsl;

pub trait PreferencesStoreGetPreference {
    fn get_value(&mut self, key: &str) -> Result<Preferences, PreferencesStoreError>;
}

#[cfg(feature = "sqlite")]
impl<'a> PreferencesStoreGetPreference
    for UserPreferencesStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn get_value(&mut self, key: &str) -> Result<Preferences, PreferencesStoreError> {
        let query = "SELECT value FROM preferences WHERE pref_key = ?";

        let preferences = diesel::sql_query(query)
            .bind::<diesel::sql_types::Text, _>(key)
            .get_result::<PreferencesModel>(self.conn)?;

        Ok(preferences.into())
    }
}

#[cfg(feature = "postgres")]
impl<'a> PreferencesStoreAddPreference
    for PreferencesStoreOperations<'a, diesel::pg::PgConnection>
{
    fn add_profile(&self, profile: Preference) -> Result<(), PreferencesStoreError> {
        let duplicate_profile = user_profile::table
            .filter(user_profile::user_id.eq(&profile.user_id))
            .first::<PreferenceModel>(self.conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| {
                PreferencesStoreError::Internal(InternalError::with_message(format!(
                    "Failed check for existing user_id {}",
                    err
                )))
            })?;

        if duplicate_profile.is_some() {
            return Err(PreferencesStoreError::ConstraintViolation(
                ConstraintViolationError::with_violation_type(ConstraintViolationType::Unique),
            ));
        }

        insert_into(user_profile::table)
            .values(PreferenceModel::from(profile))
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
