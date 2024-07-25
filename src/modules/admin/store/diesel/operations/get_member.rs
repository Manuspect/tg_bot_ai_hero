use crate::{
    modules::{
        admin::store::{diesel::models::MembersModel, Members, MembersStoreError},
        error::{InternalError, InvalidArgumentError},
    },
    schema::members,
};

use super::UserMembersStoreOperations;

use diesel::result::Error::NotFound;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

pub trait MembersStoreGetMember {
    fn get_member(&mut self, member: &str) -> Result<Members, MembersStoreError>;
}

#[cfg(feature = "sqlite")]
impl<'a> MembersStoreGetMember
    for UserMembersStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn get_member(&mut self, username: &str) -> Result<Members, MembersStoreError> {
        let member = members::table
            .filter(members::username.eq(username))
            .first::<MembersModel>(self.conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| {
                MembersStoreError::Internal(InternalError::with_message(format!(
                    "Failed check for existing username {}",
                    err
                )))
            })?
            .ok_or_else(|| {
                MembersStoreError::InvalidArgument(InvalidArgumentError::new(
                    "username".to_string(),
                    "A members for the given username does not exist".to_string(),
                ))
            })?;
        Ok(Members::from(member))
    }
}
