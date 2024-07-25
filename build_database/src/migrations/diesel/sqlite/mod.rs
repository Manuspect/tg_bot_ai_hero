//! Tools to apply database migrations for SQLite.
// use diesel::diesel_cli;
use diesel_migrations::{
    embed_migrations, EmbeddedMigrations, FileBasedMigrations, HarnessWithOutput, MigrationHarness,
};

pub const MIGRATIONS: EmbeddedMigrations =
    embed_migrations!("./src/migrations/diesel/sqlite/migrations");

use anyhow::Error;
use diesel::migration::MigrationConnection;
use diesel::sqlite::SqliteConnection;
use diesel::Connection;

use crate::build_log_debug;
use diesel_migrations::MigrationError;

/// Run all pending database migrations.
///
/// # Arguments
///
/// * `conn` - Connection to SQLite database
///
pub fn run_migrations(conn: &mut SqliteConnection) -> Result<(), Error> {
    conn.run_pending_migrations(MIGRATIONS).unwrap();
    // setup_database();
    // let migrations = FileBasedMigrations::from_path(migrations_dir).unwrap_or_else(handle_error);
    // HarnessWithOutput::write_to_stdout(conn).run_pending_migrations(migrations);
    conn.setup().unwrap();
    // diesel::migrations::run_migration_command(matches).unwrap_or_else(handle_error);
    build_log_debug!("Successfully applied Splinter SQLite migrations");
    Ok(())
}

/// Get whether there are any pending migrations
///
/// # Arguments
///
/// * `conn` - Connection to SQLite database
///
pub fn any_pending_migrations(conn: &mut SqliteConnection) -> Result<bool, Error> {
    Ok(!conn.has_pending_migration(MIGRATIONS).unwrap())
}
