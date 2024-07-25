//! Provides [`UserProfileStore`](crate::biome::profile::store::UserProfileStore) operations
//! implemented for a diesel backend

pub(super) mod add_profile;
pub(super) mod get_profile;
pub(super) mod list_profiles;
pub(super) mod remove_profile;
pub(super) mod update_profile;

pub(super) struct UserProfileStoreOperations<'a, C> {
    conn: &'a mut C,
}

impl<'a, C> UserProfileStoreOperations<'a, C>
where
    C: diesel::Connection,
{
    pub fn new(conn: &'a mut C) -> Self {
        UserProfileStoreOperations { conn }
    }
}
