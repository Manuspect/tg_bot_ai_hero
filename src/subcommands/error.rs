use std::error::Error;
use std::fmt;

use clap::Error as ClapError;

#[derive(Debug)]
pub enum CliError {
    /// A subcommand requires one or more arguments, but none were provided.
    RequiresArgs,
    /// A non-existent subcommand was specified.
    InvalidSubcommand,
    /// An error was detected by `clap`.
    ClapError(ClapError),
    /// A general error encountered by a subcommand.
    ActionError(String),
    /// The environment is not in the correct state to execute the subcommand as requested.
    EnvironmentError(String),
}

impl Error for CliError {}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CliError::RequiresArgs => write!(
                f,
                "The specified subcommand requires arguments, but none were provided"
            ),
            CliError::InvalidSubcommand => write!(f, "An invalid subcommand was specified"),
            CliError::ClapError(err) => f.write_str(&err.to_string()),
            CliError::ActionError(msg) => write!(f, "Subcommand encountered an error: {}", msg),
            CliError::EnvironmentError(msg) => f.write_str(msg),
        }
    }
}

impl From<ClapError> for CliError {
    fn from(err: ClapError) -> Self {
        Self::ClapError(err)
    }
}

impl From<build_database::error::CliError> for CliError {
    fn from(err: build_database::error::CliError) -> Self {
        match err {
            build_database::error::CliError::RequiresArgs => Self::RequiresArgs,
            build_database::error::CliError::InvalidSubcommand => Self::InvalidSubcommand,
            build_database::error::CliError::ActionError(msg) => Self::ActionError(msg),
            build_database::error::CliError::EnvironmentError(msg) => Self::EnvironmentError(msg),
        }
    }
}
