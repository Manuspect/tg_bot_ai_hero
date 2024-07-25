#![doc(hidden)]

use std::fmt::{Debug, Display};
use std::mem::ManuallyDrop;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::thread::{Builder as ThreadBuilder, JoinHandle};

use anyhow::Error;
use build_database::build_database::sqlite_migrations;
use tokio::runtime::Handle;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::Notify;

use diesel::r2d2::{ConnectionManager, Pool};

use crate::store::{sqlite, StoreFactory};

pub enum ConnectionPool {
    #[cfg(feature = "postgres")]
    Postgres {
        pool: Pool<ConnectionManager<diesel::pg::PgConnection>>,
    },
    #[cfg(feature = "sqlite")]
    Sqlite {
        pool: Arc<RwLock<Pool<ConnectionManager<diesel::SqliteConnection>>>>,
    },
    // This variant is only enabled to such that the compiler does not complain.  It is never
    // constructed.
    #[cfg(not(any(feature = "database-postgres", feature = "database-sqlite")))]
    #[allow(dead_code)]
    Unsupported,
}

/// The possible connection types and identifiers for a `StoreFactory`
pub enum ConnectionUri {
    Memory,
    #[cfg(feature = "postgres")]
    Postgres(String),
    #[cfg(feature = "sqlite")]
    Sqlite(String),
}

impl Display for ConnectionUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            ConnectionUri::Memory => "memory",
            #[cfg(feature = "sqlite")]
            ConnectionUri::Sqlite(sqlite) => sqlite,
            #[cfg(feature = "postgres")]
            ConnectionUri::Postgres(pg) => pg,
        };
        write!(f, "{}", string)
    }
}

impl FromStr for ConnectionUri {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "memory" => Ok(ConnectionUri::Memory),
            #[cfg(feature = "postgres")]
            _ if s.starts_with("postgres://") => Ok(ConnectionUri::Postgres(s.into())),
            #[cfg(feature = "sqlite")]
            _ => Ok(ConnectionUri::Sqlite(s.into())),
            #[cfg(not(feature = "sqlite"))]
            _ => Err(anyhow!(
                "s".to_string(),
                format!("No compatible connection type: {}", s),
            )),
        }
    }
}

pub fn create_connection_pool(connection_uri: &ConnectionUri) -> Result<ConnectionPool, Error> {
    log::info!("create_connection_pool: {}", connection_uri);
    match connection_uri {
        #[cfg(feature = "postgres")]
        ConnectionUri::Postgres(url) => {
            let pool = postgres::create_postgres_connection_pool(url)?;
            Ok(ConnectionPool::Postgres { pool })
        }
        #[cfg(feature = "sqlite")]
        ConnectionUri::Sqlite(conn_str) => {
            let pool = sqlite::create_sqlite_connection_pool_with_write_exclusivity(conn_str)?;
            Ok(ConnectionPool::Sqlite { pool })
        }
        #[cfg(feature = "sqlite")]
        ConnectionUri::Memory => {
            let pool = sqlite::create_sqlite_connection_pool_with_write_exclusivity(":memory:")?;
            Ok(ConnectionPool::Sqlite { pool })
        }
        #[cfg(not(feature = "sqlite"))]
        ConnectionUri::Memory => Err(anyhow!("Unsupported connection pool type: memory".into(),)),
    }
}

/// Creates a `StoreFactory` backed by the given connection
///
/// # Arguments
///
/// * `connection_uri` - The identifier of the storage connection that will be used by all stores
///   created by the resulting factory
pub fn create_store_factory(
    connection_pool: &ConnectionPool,
) -> Result<Box<dyn StoreFactory>, Error> {
    match connection_pool {
        #[cfg(feature = "postgres")]
        ConnectionPool::Postgres { pool } => {
            Ok(Box::new(postgres::PgStoreFactory::new(pool.clone())))
        }
        #[cfg(feature = "sqlite")]
        ConnectionPool::Sqlite { pool } => Ok(Box::new(
            sqlite::SqliteStoreFactory::new_with_write_exclusivity(pool.clone()),
        )),
        #[cfg(not(any(feature = "database-postgres", feature = "database-sqlite")))]
        ConnectionPool::Unsupported => Err(anyhow!(
            "Connection pools are unavailable in this configuration".to_string(),
        )),
    }
}

pub(crate) trait DatabaseProvider {
    fn provide_db(&self) -> Result<ConnectionPool, Error>;
}

pub(crate) struct InMemDatabaseProvider {
    db_url: ConnectionUri,
}

impl InMemDatabaseProvider {
    pub fn new(path: &str) -> Self {
        Self {
            db_url: path
                .parse()
                .map_err(|e| anyhow!(format!("Invalid database URL provided: {}", e)))
                .unwrap(),
        }
    }
}

impl DatabaseProvider for InMemDatabaseProvider {
    /// This function a database connection pool.
    fn provide_db(&self) -> Result<ConnectionPool, Error> {
        let connection_pool = create_connection_pool(&self.db_url)
            .map_err(|err| anyhow!(format!("Failed to initialize connection pool: {}", err)))?;

        Ok(connection_pool)
    } // end fn create_app_state

    // fn provide_db(&self) -> Result<Connection, Error> {
    //     // let conn = Connection::open_in_memory()?;
    //     let conn = &mut SqliteConnection::establish(":memory:").unwrap();
    //     Ok(conn)
    // }
}

pub(crate) struct FileDatabaseProvider {
    db_url: ConnectionUri,
}

impl FileDatabaseProvider {
    pub fn new(path: &str) -> Self {
        Self {
            db_url: path
                .parse()
                .map_err(|e| anyhow!(format!("Invalid database URL provided: {}", e)))
                .unwrap(),
        }
    }
}

impl DatabaseProvider for FileDatabaseProvider {
    /// This function a database connection pool.
    fn provide_db(&self) -> Result<ConnectionPool, Error> {
        match &self.db_url {
            #[cfg(feature = "postgres")]
            ConnectionUri::Postgres(url) => postgres::postgres_migrations(&url)?,
            #[cfg(feature = "sqlite")]
            ConnectionUri::Sqlite(connection_string) => {
                sqlite_migrations(connection_string.clone())?
            }
            ConnectionUri::Memory => {}
        }

        let connection_pool = create_connection_pool(&self.db_url)
            .map_err(|err| anyhow!(format!("Failed to initialize connection pool: {}", err)))?;

        Ok(connection_pool)
    } // end fn create_app_state

    // fn provide_db(&self) -> Result<ConnectionPool, Error> {
    //     let conn = ConnectionPool::open(&self.path)?;
    //     Ok(conn)
    // }
}

pub(crate) struct DatabaseManager {
    inner: Arc<DatabaseManagerInner>,
}

impl DatabaseManager {
    pub fn with_db_provider<P>(provider: P) -> Result<Self, Error>
    where
        P: DatabaseProvider,
    {
        let conn = provider.provide_db()?;
        let (work_tx, work_rx) = channel(10);
        let shutdown_notify = Arc::new(Notify::new());

        let rt_handle = Handle::current();

        let db_thread = DatabaseThread::new(conn, rt_handle, work_rx, Arc::clone(&shutdown_notify));
        let join_handle = ManuallyDrop::new(db_thread.start());

        Ok(Self {
            inner: Arc::new(DatabaseManagerInner {
                join_handle,
                work_tx,
                shutdown_notify,
            }),
        })
    }

    pub async fn enqueue_work<F>(&self, f: F) -> Result<(), Error>
    where
        F: FnOnce(&mut ConnectionPool) + Send + 'static,
    {
        let work = AnyDatabaseThreadWork::new(f);
        self.inner
            .work_tx
            .send(Box::new(work))
            .await
            .map_err(|err| anyhow!(err.to_string()))?;

        Ok(())
    }

    pub async fn query<F, R>(&self, f: F) -> Result<R, Error>
    where
        F: FnOnce(&mut ConnectionPool) -> R + Send + 'static,
        R: Send + Debug + 'static,
    {
        let (res_tx, res_rx) = tokio::sync::oneshot::channel();
        self.enqueue_work(move |conn| {
            let res = f(conn);
            res_tx.send(res).unwrap();
        })
        .await?;

        res_rx.await.map_err(|err| anyhow!(err.to_string()))
    }
}

impl Clone for DatabaseManager {
    fn clone(&self) -> Self {
        DatabaseManager {
            inner: Arc::clone(&self.inner),
        }
    }
}

struct DatabaseManagerInner {
    join_handle: ManuallyDrop<JoinHandle<()>>,
    work_tx: Sender<Box<dyn DatabaseThreadWork>>,
    shutdown_notify: Arc<Notify>,
}

impl Drop for DatabaseManagerInner {
    fn drop(&mut self) {
        // Gracefully shutdown the database thread.
        self.shutdown_notify.notify_one();
        let join_handle = unsafe { ManuallyDrop::take(&mut self.join_handle) };
        join_handle.join().unwrap();

        debug!("Database thread has shutdown");
    }
}

struct DatabaseThread {
    conn: ConnectionPool,
    rt_handle: Handle,
    work_rx: Receiver<Box<dyn DatabaseThreadWork>>,
    shutdown_notify: Arc<Notify>,
    shutdown: bool,
}

impl DatabaseThread {
    fn new(
        conn: ConnectionPool,
        rt_handle: Handle,
        work_rx: Receiver<Box<dyn DatabaseThreadWork>>,
        shutdown_notify: Arc<Notify>,
    ) -> Self {
        Self {
            conn,
            rt_handle,
            work_rx,
            shutdown_notify,
            shutdown: false,
        }
    }

    fn start(self) -> JoinHandle<()> {
        ThreadBuilder::new()
            .name("DatabaseThread".to_owned())
            .spawn(move || {
                let mut thread = self;
                let handle = thread.rt_handle.clone();
                handle.block_on(async move {
                    thread.run_loop().await;
                });
            })
            .unwrap()
    }

    async fn run_loop(&mut self) {
        while !self.shutdown {
            self.poll_once().await;
        }
    }

    async fn poll_once(&mut self) {
        tokio::select! {
            _ = self.shutdown_notify.notified() => {
                self.shutdown = true;
            },
            maybe_work = self.work_rx.recv() => {
                if let Some(mut work) = maybe_work {
                    work.perform(&mut self.conn);
                } else {
                    // No more work to perform, the thread is requested to terminate.
                    self.shutdown = true;
                }
            }
        };
    }
}

trait DatabaseThreadWork: Send {
    fn perform(&mut self, conn: &mut ConnectionPool);
}

struct AnyDatabaseThreadWork<F>
where
    F: FnOnce(&mut ConnectionPool) + Send,
{
    f: Option<F>,
}

impl<F> AnyDatabaseThreadWork<F>
where
    F: FnOnce(&mut ConnectionPool) + Send,
{
    fn new(f: F) -> Self {
        Self { f: Some(f) }
    }
}

impl<F> Debug for AnyDatabaseThreadWork<F>
where
    F: FnOnce(&mut ConnectionPool) + Send,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DatabaseThreadWork")
    }
}

impl<F> DatabaseThreadWork for AnyDatabaseThreadWork<F>
where
    F: FnOnce(&mut ConnectionPool) + Send,
{
    fn perform(&mut self, conn: &mut ConnectionPool) {
        let f = self.f.take().unwrap();
        f(conn)
    }
}
