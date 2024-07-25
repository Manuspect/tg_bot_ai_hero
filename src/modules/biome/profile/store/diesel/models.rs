use serde::Deserialize;

use crate::modules::biome::profile::store::Profile;

#[derive(Insertable, Selectable, Queryable, Identifiable, PartialEq, Eq, Debug, Deserialize)]
#[diesel(table_name = crate::schema::user_profile)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[primary_key(user_id)]
pub struct ProfileModel {
    pub user_id: Option<String>,
    pub subject: String,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub email: Option<String>,
    pub picture: Option<String>,
}

impl From<Profile> for ProfileModel {
    fn from(profile: Profile) -> Self {
        ProfileModel {
            user_id: profile.user_id,
            subject: profile.subject,
            name: profile.name,
            given_name: profile.given_name,
            family_name: profile.family_name,
            email: profile.email,
            picture: profile.picture,
        }
    }
}
