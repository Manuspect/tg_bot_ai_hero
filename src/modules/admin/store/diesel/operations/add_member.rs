use crate::{
    modules::{
        admin::store::{diesel::models::MembersModel, Members, MembersStoreError},
        error::{ConstraintViolationError, ConstraintViolationType, InternalError},
    },
    schema::members,
};

use super::UserMembersStoreOperations;

use diesel::{dsl::insert_into, result::Error::NotFound};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

pub trait MembersStoreAddMember {
    fn add_member(&mut self, member: Members) -> Result<(), MembersStoreError>;
}

#[cfg(feature = "sqlite")]
impl<'a> MembersStoreAddMember
    for UserMembersStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn add_member(&mut self, members: Members) -> Result<(), MembersStoreError> {
        let duplicate_member = members::table
            .filter(members::username.eq(&members.username))
            .first::<MembersModel>(self.conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| {
                MembersStoreError::Internal(InternalError::with_message(format!(
                    "Failed check for existing user_id {}",
                    err
                )))
            })?;

        if duplicate_member.is_some() {
            return Err(MembersStoreError::ConstraintViolation(
                ConstraintViolationError::with_violation_type(ConstraintViolationType::Unique),
            ));
        }

        insert_into(members::table)
            .values(MembersModel::from(members.clone()))
            .on_conflict(members::username)
            .do_update()
            .set(MembersModel::from(members.clone()))
            .execute(self.conn)
            .map(|_| ())
            .map_err(|_| {
                MembersStoreError::Internal(InternalError::with_message(
                    "Failed to add credentials".to_string(),
                ))
            })?;
        Ok(())
    }
}
