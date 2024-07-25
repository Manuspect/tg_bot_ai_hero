use serde::Deserialize;

use crate::modules::prefs::store::Preferences;

use crate::schema::preferences;

// #[derive(
//     Default, Clone, Debug, Queryable, Selectable, Identifiable, Insertable, Serialize, Deserialize,
// )]
#[derive(
    Insertable,
    Selectable,
    Queryable,
    QueryableByName,
    Identifiable,
    PartialEq,
    Eq,
    Debug,
    Deserialize,
    AsChangeset,
)]
#[table_name = "preferences"]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[primary_key(pref_key)]
pub struct PreferencesModel {
    pub pref_key: String,
    pub value: Option<String>,
}

impl From<Preferences> for PreferencesModel {
    fn from(preferences: Preferences) -> Self {
        PreferencesModel {
            pref_key: preferences.key,
            value: preferences.value,
        }
    }
}
