use crate::{
    modules::{
        admin::store::{diesel::models::MembersModel, MembersStoreError},
        error::{InternalError, InvalidArgumentError},
    },
    schema::members,
};

use super::UserMembersStoreOperations;

use diesel::{delete, result::Error::NotFound};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

pub trait MembersStoreDeleteMember {
    fn delete_member(&mut self, member: &str) -> Result<(), MembersStoreError>;
}

#[cfg(feature = "sqlite")]
impl<'a> MembersStoreDeleteMember
    for UserMembersStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn delete_member(&mut self, username: &str) -> Result<(), MembersStoreError> {
        let profile = members::table
            .filter(members::username.eq(username))
            .first::<MembersModel>(self.conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| {
                MembersStoreError::Internal(InternalError::with_message(format!(
                    "Failed check for existing username {}",
                    err
                )))
            })?;
        if profile.is_none() {
            return Err(MembersStoreError::InvalidArgument(
                InvalidArgumentError::new(
                    "username".to_string(),
                    "A member for the given username does not exist".to_string(),
                ),
            ));
        }

        delete(members::table.filter(members::username.eq(username)))
            .execute(self.conn)
            .map(|_| ())
            .map_err(|err| {
                MembersStoreError::Internal(InternalError::with_message(format!(
                    "Failed check for existing username {}",
                    err
                )))
            })?;
        Ok(())
    }
}
