use std::sync::{Arc, RwLock};

use diesel::r2d2::{ConnectionManager, Pool};

#[cfg(any(
    any(feature = "postgres", feature = "sqlite"),
    all(feature = "diesel", feature = "registry")
))]
use crate::modules::error::InternalError;

pub enum ConnectionPool<C: diesel::r2d2::R2D2Connection + 'static> {
    Normal(Pool<ConnectionManager<C>>),
    WriteExclusive(Arc<RwLock<Pool<ConnectionManager<C>>>>),
}

#[cfg(any(
    any(feature = "postgres", feature = "sqlite"),
    all(feature = "diesel", feature = "registry")
))]
macro_rules! conn {
    ($pool:ident) => {
        $pool
            .get()
            .map_err(|e| InternalError::with_message(e.to_string()))
    };
}

#[cfg(any(
    any(feature = "postgres", feature = "sqlite"),
    all(feature = "diesel", feature = "registry")
))]
impl<C: diesel::r2d2::R2D2Connection> ConnectionPool<C> {
    pub fn execute_write<F, T, E>(&self, f: F) -> Result<T, E>
    where
        F: FnOnce(&mut C) -> Result<T, E>,
        E: From<InternalError>,
    {
        match self {
            Self::Normal(pool) => f(&mut *conn!(pool)?),
            Self::WriteExclusive(locked_pool) => locked_pool
                .write()
                .map_err(|_| {
                    InternalError::with_message("Connection pool rwlock is poisoned".into()).into()
                })
                .and_then(|pool| f(&mut *conn!(pool)?)),
        }
    }

    pub fn execute_read<F, T, E>(&self, f: F) -> Result<T, E>
    where
        F: FnOnce(&mut C) -> Result<T, E>,
        E: From<InternalError>,
    {
        match self {
            Self::Normal(pool) => f(&mut *conn!(pool)?),
            Self::WriteExclusive(locked_pool) => locked_pool
                .read()
                .map_err(|_| {
                    InternalError::with_message("Connection pool rwlock is poisoned".into()).into()
                })
                .and_then(|pool| f(&mut *conn!(pool)?)),
        }
    }
}

impl<C: diesel::r2d2::R2D2Connection> Clone for ConnectionPool<C> {
    fn clone(&self) -> Self {
        match self {
            Self::Normal(pool) => Self::Normal(pool.clone()),
            Self::WriteExclusive(locked_pool) => Self::WriteExclusive(locked_pool.clone()),
        }
    }
}

impl<C: diesel::r2d2::R2D2Connection> From<Pool<ConnectionManager<C>>> for ConnectionPool<C> {
    fn from(pool: Pool<ConnectionManager<C>>) -> Self {
        Self::Normal(pool)
    }
}

impl<C: diesel::r2d2::R2D2Connection> From<Arc<RwLock<Pool<ConnectionManager<C>>>>>
    for ConnectionPool<C>
{
    fn from(pool: Arc<RwLock<Pool<ConnectionManager<C>>>>) -> Self {
        Self::WriteExclusive(pool)
    }
}

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use super::*;

    use diesel::connection::SimpleConnection;
    use diesel::prelude::*;

    /// Given a SqliteConnection pool
    /// 1. Create a simple table
    /// 2. Wrap the diesel pool in the ConnectionPool with an Arc-RwLock combo
    /// 2. Spawn N threads and pass a clone of the wrapped pool to each
    /// 3. In each thread, perform M reads and writes; this helps increase the likelihood that
    ///    there will be a write collision.
    /// 4. Validate that all threads exit successfully.
    #[test]
    fn test_multithreaded_read_write() -> Result<(), Box<dyn std::error::Error>> {
        let pool = create_connection_pool("test_multithreaded_read_write")?;

        pool.get()?.batch_execute(
            r#"
            CREATE TABLE test_table (
                id TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            "#,
        )?;

        let conn_pool = ConnectionPool::from(Arc::new(RwLock::new(pool)));

        let (tx, rx) = std::sync::mpsc::channel();
        let thread_count = 10;
        for t in 0..thread_count {
            let t_pool = conn_pool.clone();
            let signaller = tx.clone();

            std::thread::Builder::new()
                .name(format!("test_multithreaded_read_write-{}", t))
                .spawn(move || {
                    for i in 0..10 {
                        let id = format!("{}-{}", t, i);
                        t_pool
                            .execute_write::<_, (), InternalError>(|conn| {
                                diesel::sql_query(
                                    "INSERT INTO test_table (id, value) VALUES (?, ?)",
                                )
                                .bind::<diesel::sql_types::Text, _>(&id)
                                .bind::<diesel::sql_types::Text, _>("test")
                                .execute(conn)
                                .map_err(|e| InternalError::with_message(e.to_string()))?;

                                Ok(())
                            })
                            .unwrap();

                        let value = t_pool
                            .execute_read::<_, _, InternalError>(|conn| {
                                diesel::sql_query("SELECT * FROM test_table WHERE id = ?")
                                    .bind::<diesel::sql_types::Text, _>(&id)
                                    .get_results::<LookupData>(conn)
                                    .map_err(|e| InternalError::with_message(e.to_string()))
                            })
                            .unwrap();

                        assert_eq!(id, value[0]._id);
                        assert_eq!("test", &value[0]._value);
                    }

                    signaller.send(()).unwrap();
                })
                .unwrap();
        }
        drop(tx);

        assert_eq!(rx.iter().count(), thread_count);

        Ok(())
    }

    #[derive(QueryableByName)]
    struct LookupData {
        #[column_name = "id"]
        #[sql_type = "diesel::sql_types::Text"]
        _id: String,

        #[column_name = "value"]
        #[sql_type = "diesel::sql_types::Text"]
        _value: String,
    }

    fn create_connection_pool(
        test_name: &str,
    ) -> Result<Pool<ConnectionManager<diesel::SqliteConnection>>, Box<dyn std::error::Error>> {
        let connection_manager = ConnectionManager::<diesel::SqliteConnection>::new(&format!(
            "file:{}?mode=memory&cache=shared",
            test_name
        ));
        let pool = Pool::builder().build(connection_manager)?;

        Ok(pool)
    }
}
