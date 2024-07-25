use crate::{
    modules::{
        error::{ConstraintViolationError, ConstraintViolationType, InternalError},
        stats::store::{diesel::models::TokenUsageModel, TokenUsage, TokenUsageStoreError},
    },
    schema::token_usage,
};

use super::UserTokenUsageStoreOperations;

use diesel::{dsl::insert_into, result::Error::NotFound, BoolExpressionMethods};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

pub trait TokenUsageStoreAddTokenUsage {
    fn add_token_usage(&mut self, profile: TokenUsage) -> Result<(), TokenUsageStoreError>;
}

#[cfg(feature = "sqlite")]
impl<'a> TokenUsageStoreAddTokenUsage
    for UserTokenUsageStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn add_token_usage(&mut self, token_usage: TokenUsage) -> Result<(), TokenUsageStoreError> {
        let duplicate_profile = token_usage::table
            .filter(
                token_usage::user_id
                    .eq(&token_usage.user_id)
                    .and(token_usage::time.eq(&token_usage.time)),
            )
            .first::<TokenUsageModel>(self.conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| {
                TokenUsageStoreError::Internal(InternalError::with_message(format!(
                    "Failed check for existing user_id {}",
                    err
                )))
            })?;

        if duplicate_profile.is_some() {
            return Err(TokenUsageStoreError::ConstraintViolation(
                ConstraintViolationError::with_violation_type(ConstraintViolationType::Unique),
            ));
        }

        insert_into(token_usage::table)
            .values(TokenUsageModel::from(token_usage.clone()))
            .on_conflict((token_usage::time, token_usage::user_id))
            .do_update()
            .set(TokenUsageModel::from(token_usage.clone()))
            .execute(self.conn)
            .map(|_| ())
            .map_err(|_| {
                TokenUsageStoreError::Internal(InternalError::with_message(
                    "Failed to add credentials".to_string(),
                ))
            })?;
        Ok(())
    }
}

#[cfg(feature = "postgres")]
impl<'a> TokenUsageStoreAddTokenUsage for TokenUsageStoreOperations<'a, diesel::pg::PgConnection> {
    fn add_profile(&self, profile: TokenUsage) -> Result<(), TokenUsageStoreError> {
        let duplicate_profile = user_profile::table
            .filter(user_profile::user_id.eq(&profile.user_id))
            .first::<TokenUsageModel>(self.conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| {
                TokenUsageStoreError::Internal(InternalError::with_message(format!(
                    "Failed check for existing user_id {}",
                    err
                )))
            })?;

        if duplicate_profile.is_some() {
            return Err(TokenUsageStoreError::ConstraintViolation(
                ConstraintViolationError::with_violation_type(ConstraintViolationType::Unique),
            ));
        }

        insert_into(user_profile::table)
            .values(TokenUsageModel::from(profile))
            .execute(self.conn)
            .map(|_| ())
            .map_err(|_| {
                TokenUsageStoreError::Internal(InternalError::with_message(
                    "Failed to add credentials".to_string(),
                ))
            })?;
        Ok(())
    }
}
