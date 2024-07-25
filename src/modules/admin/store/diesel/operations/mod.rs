//! Provides [`MembersStore`](crate::biome::profile::store::MembersStore) operations
//! implemented for a diesel backend

pub(super) mod add_member;
pub(super) mod delete_member;
pub(super) mod get_member;
pub(super) struct UserMembersStoreOperations<'a, C> {
    conn: &'a mut C,
}

impl<'a, C> UserMembersStoreOperations<'a, C>
where
    C: diesel::Connection,
{
    pub fn new(conn: &'a mut C) -> Self {
        UserMembersStoreOperations { conn }
    }
}
