//! Provides [`TokenUsageStore`](crate::biome::profile::store::TokenUsageStore) operations
//! implemented for a diesel backend

pub(super) mod add_token_usage;
pub(super) mod get_token_usage;

pub(super) struct UserTokenUsageStoreOperations<'a, C> {
    conn: &'a mut C,
}

impl<'a, C> UserTokenUsageStoreOperations<'a, C>
where
    C: diesel::Connection,
{
    pub fn new(conn: &'a mut C) -> Self {
        UserTokenUsageStoreOperations { conn }
    }
}
