//! Tools to apply database migrations for Postgres.

embed_migrations!("./src/migrations/diesel/postgres/migrations");

use diesel::pg::PgConnection;
use diesel::Connection;
use diesel_migrations::MigrationConnection;

use crate::error::InternalError;

/// Run all pending database migrations.
///
/// # Arguments
///
/// * `conn` - Connection to PostgreSQL database
///
pub fn run_migrations(conn: &PgConnection) -> Result<(), InternalError> {
    embedded_migrations::run(conn).map_err(|err| InternalError::from_source(Box::new(err)))?;

    debug!("Successfully applied Splinter PostgreSQL migrations");

    Ok(())
}

/// Get whether there are any pending migrations
///
/// # Arguments
///
/// * `conn` - Connection to PostgreSQL database
///
pub fn any_pending_migrations(conn: &PgConnection) -> Result<bool, InternalError> {
    let current_version = conn.latest_run_migration_version().unwrap_or(None);

    // Diesel 1.4 only allows access to the list of migrations via attempting
    // to run the migrations, so we'll do that in a test transaction.
    let latest_version =
        conn.test_transaction::<Result<Option<String>, InternalError>, (), _>(|| {
            Ok(match embedded_migrations::run(conn) {
                Ok(_) => conn
                    .latest_run_migration_version()
                    .map_err(|err| InternalError::from_source(Box::new(err))),
                Err(err) => Err(InternalError::from_source(Box::new(err))),
            })
        })?;

    Ok(current_version == latest_version)
}
