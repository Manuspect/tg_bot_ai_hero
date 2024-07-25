use serde::Deserialize;

use crate::modules::stats::store::TokenUsage;

use crate::schema::token_usage;

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
#[table_name = "token_usage"]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[primary_key(user_id, time)]
pub struct TokenUsageModel {
    pub user_id: String,
    pub time: chrono::NaiveDateTime,
    pub tokens: i64,
}

impl From<TokenUsage> for TokenUsageModel {
    fn from(token_usage: TokenUsage) -> Self {
        TokenUsageModel {
            user_id: token_usage.user_id,
            time: token_usage.time,
            tokens: token_usage.tokens,
        }
    }
}
