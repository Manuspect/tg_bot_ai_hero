// pub mod factory;
#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "sqlite")]
mod sqlite;

// mod state;
mod stores;
#[cfg(feature = "upgrade")]
mod upgrade;

use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{env, fs};

use dotenvy::dotenv;

#[cfg(not(feature = "sqlite"))]
use self::postgres::get_default_database;
#[cfg(feature = "sqlite")]
pub use self::sqlite::{get_default_database, sqlite_migrations};
// pub use self::state::StateMigrateAction;
#[cfg(feature = "upgrade")]
pub use self::upgrade::UpgradeAction;
use crate::build_log_info;
use crate::error::CliError;

const HOME_ENV: &str = "WORK_HOME";
const STATE_DIR_ENV: &str = "STATE_DIR";
const DEFAULT_STATE_DIR: &str = "/var/lib/splinter";

pub struct Migrate;

impl Migrate {
    pub fn run<'a>(url: Option<String>) -> Result<(), CliError> {
        let url = match url {
            Some(url) => url.to_owned(),
            None => get_default_database()?,
        };
        // std::env::set_var("DATABASE_URL", url.clone());
        build_log_info!("url: {}", url);
        match ConnectionUri::from_str(&url)? {
            #[cfg(feature = "postgres")]
            ConnectionUri::Postgres(url) => postgres::postgres_migrations(&url)?,
            #[cfg(feature = "sqlite")]
            ConnectionUri::Sqlite(connection_string) => sqlite_migrations(connection_string)?,
        }

        Ok(())
    }
}

/// The possible connection types and identifiers passed to the migrate command
pub enum ConnectionUri {
    #[cfg(feature = "postgres")]
    Postgres(String),
    #[cfg(feature = "sqlite")]
    Sqlite(String),
}

impl std::fmt::Display for ConnectionUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            #[cfg(feature = "postgres")]
            ConnectionUri::Postgres(pg) => pg,
            #[cfg(feature = "sqlite")]
            ConnectionUri::Sqlite(sqlite) => sqlite,
        };
        f.write_str(string)
    }
}

impl FromStr for ConnectionUri {
    type Err = CliError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            // check specifically so it does not pass to sqlite
            "memory" => Err(CliError::ActionError(format!(
                "No compatible connection type: {}",
                s
            ))),
            #[cfg(feature = "postgres")]
            _ if s.starts_with("postgres://") => Ok(ConnectionUri::Postgres(s.into())),
            #[cfg(feature = "sqlite")]
            _ => Ok(ConnectionUri::Sqlite(s.into())),
            #[cfg(not(feature = "sqlite"))]
            _ => Err(CliError::ActionError(format!(
                "No compatible connection type: {}",
                s
            ))),
        }
    }
}

/// Represents the SplinterEnvironment data
#[derive(Debug)]
struct SplinterEnvironment {
    state_dir: Option<String>,
    home_dir: Option<String>,
    default_dir: &'static str,
}

impl SplinterEnvironment {
    pub fn load() -> Self {
        dotenv().ok();
        SplinterEnvironment {
            state_dir: env::var(STATE_DIR_ENV).ok(),
            home_dir: env::var(HOME_ENV).ok(),
            default_dir: DEFAULT_STATE_DIR,
        }
    }

    fn try_canonicalize<P: Into<PathBuf>>(dir: P) -> PathBuf {
        let dir: PathBuf = dir.into();
        fs::canonicalize(dir.clone()).unwrap_or(dir)
    }

    /// Returns the path to the state directory
    ///
    /// If `STATE_DIR` is set, returns `STATE_DIR`.
    /// If `WORK_HOME` is set, returns `WORK_HOME/data`.
    /// Otherwise, returns the default directory `/var/lib/splinter`
    pub fn get_state_path(&self) -> PathBuf {
        build_log_info!("{:?}", self.state_dir);
        build_log_info!("{:?}", self.home_dir);
        build_log_info!("{:?}", self.default_dir);
        if let Some(state_dir) = self.state_dir.as_ref() {
            Self::try_canonicalize(PathBuf::from(&state_dir))
        } else if let Some(home_dir) = self.home_dir.as_ref() {
            Self::try_canonicalize(Path::new(&home_dir).join("data"))
        } else {
            Self::try_canonicalize(PathBuf::from(&self.default_dir))
        }
    }
}
