//! Provides sql migration scripts and methods for executing
//! migrations.
//!
//! ```ignore
//! use migrations::run_postgres_migrations;
//! use diesel::{pg::PgConnection};
//!
//! let connection = PgConnection::establish(
//!      "postgres://admin:admin@localhost:5432/splinterd").unwrap();
//!
//! run_postgres_migrations(&connection).unwrap();
//!
//! ```

#[cfg(feature = "diesel")]
mod diesel;

#[cfg(feature = "postgres")]
pub use self::diesel::postgres::any_pending_migrations as any_pending_postgres_migrations;
#[cfg(feature = "postgres")]
pub use self::diesel::postgres::run_migrations as run_postgres_migrations;
#[cfg(feature = "sqlite")]
pub use self::diesel::sqlite::any_pending_migrations as any_pending_sqlite_migrations;
#[cfg(feature = "sqlite")]
pub use self::diesel::sqlite::run_migrations as run_sqlite_migrations;
