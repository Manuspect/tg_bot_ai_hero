use serde::Deserialize;

use crate::{modules::admin::store::Members, schema::members};

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
#[table_name = "members"]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[primary_key(username)]
pub struct MembersModel {
    pub username: String,
    pub disabled: i32,
    pub created_at: chrono::NaiveDateTime,
}

impl From<Members> for MembersModel {
    fn from(members: Members) -> Self {
        MembersModel {
            username: members.username,
            disabled: members.disabled,
            created_at: members.created_at,
        }
    }
}
