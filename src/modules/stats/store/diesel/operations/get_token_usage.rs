use diesel::{prelude::*, result::Error::NotFound};

use crate::{
    modules::{
        error::{InternalError, InvalidArgumentError},
        stats::store::{diesel::models::TokenUsageModel, TokenUsage, TokenUsageStoreError},
    },
    schema::token_usage,
};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

use super::UserTokenUsageStoreOperations;

pub trait UserTokenUsageStoreGetTokenUsage {
    fn get_profile(
        &mut self,
        user_id: &str,
        time: &chrono::NaiveDateTime,
    ) -> Result<TokenUsage, TokenUsageStoreError>;
    fn query_usage_of_user(&mut self, user_id: &str) -> Result<i64, TokenUsageStoreError>;
    fn query_total_usage(&mut self) -> Result<i64, TokenUsageStoreError>;
}

#[cfg(feature = "sqlite")]
impl<'a> UserTokenUsageStoreGetTokenUsage
    for UserTokenUsageStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn get_profile(
        &mut self,
        user_id: &str,
        time: &chrono::NaiveDateTime,
    ) -> Result<TokenUsage, TokenUsageStoreError> {
        let token_usage = token_usage::table
            .filter(
                token_usage::user_id
                    .eq(user_id)
                    .and(token_usage::time.eq(time)),
            )
            .first::<TokenUsageModel>(self.conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| {
                TokenUsageStoreError::Internal(InternalError::with_message(format!(
                    "Failed check for existing user_id {}",
                    err
                )))
            })?
            .ok_or_else(|| {
                TokenUsageStoreError::InvalidArgument(InvalidArgumentError::new(
                    "user_id".to_string(),
                    "A token_usage for the given user_id does not exist".to_string(),
                ))
            })?;
        Ok(TokenUsage::from(token_usage))
    }

    fn query_usage_of_user(&mut self, user_id: &str) -> Result<i64, TokenUsageStoreError> {
        #[derive(QueryableByName, Clone)]
        struct UsageQueryResult {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            tokens_amount: i64,
        }

        let query = "SELECT SUM(tokens) AS tokens_amount FROM token_usage WHERE user_id = ?";

        let usage_query_result = diesel::sql_query(query)
            .bind::<diesel::sql_types::Text, _>(user_id)
            .load::<UsageQueryResult>(self.conn)?;

        let mut available_balance = None;
        for usage in usage_query_result {
            available_balance = Some(usage.tokens_amount)
        }
        Ok(available_balance.ok_or_else(|| {
            TokenUsageStoreError::InvalidArgument(InvalidArgumentError::new(
                "user_id".to_string(),
                "A token_usage for the given user_id does not exist".to_string(),
            ))
        })?)
    }

    fn query_total_usage(&mut self) -> Result<i64, TokenUsageStoreError> {
        #[derive(QueryableByName, Clone)]
        struct UsageQueryResult {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            tokens_amount: i64,
        }

        let query = "SELECT SUM(tokens) AS tokens_amount FROM token_usage";

        let usage_query_result = diesel::sql_query(query).load::<UsageQueryResult>(self.conn)?;

        let mut available_balance = None;
        for usage in usage_query_result {
            available_balance = Some(usage.tokens_amount)
        }
        Ok(available_balance.ok_or_else(|| {
            TokenUsageStoreError::InvalidArgument(InvalidArgumentError::new(
                "user_id".to_string(),
                "A token_usage for the given user_id does not exist".to_string(),
            ))
        })?)
    }
}
