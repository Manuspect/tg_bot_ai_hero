//! Provides [`PreferencesStore`](crate::biome::profile::store::PreferencesStore) operations
//! implemented for a diesel backend

pub(super) mod get_value;
pub(super) mod set_value;

pub(super) struct UserPreferencesStoreOperations<'a, C> {
    conn: &'a mut C,
}

impl<'a, C> UserPreferencesStoreOperations<'a, C>
where
    C: diesel::Connection,
{
    pub fn new(conn: &'a mut C) -> Self {
        UserPreferencesStoreOperations { conn }
    }
}
