//! Common set of basic errors used throughout the library.
//!
//! The errors in this module are intended to be used by themselves or as part of a more complex
//! error `enum`.
//!
//! # Examples
//!
//! ## Returning an Error from a Function
//!
//! A function may return an error such as `InternalError` by itself.
//!
//! ```
//! use std::fs;
//!
//! use splinter::error::InternalError;
//!
//! fn check_path(path: &str) -> Result<bool, InternalError> {
//!     let metadata = fs::metadata(path).map_err(|e| InternalError::from_source(Box::new(e)))?;
//!     Ok(metadata.is_file())
//! }
//! ```
//!
//! ## Constructing Complex Errors
//!
//! Errors such as `InternalError` may be used to construct more complicated errors by defining
//! an `enum`.
//!
//! ```
//! use std::error;
//! use std::fmt;
//! use std::fs;
//!
//! use splinter::error::InternalError;
//!
//! #[derive(Debug)]
//! enum MyError {
//!     InternalError(InternalError),
//!     MissingFilenameExtension,
//! }
//!
//! impl error::Error for MyError {}
//!
//! impl fmt::Display for MyError {
//!     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//!         match self {
//!             MyError::InternalError(e) => write!(f, "{}", e),
//!             MyError::MissingFilenameExtension => write!(f, "Missing filename extension"),
//!         }
//!     }
//! }
//!
//! fn check_path(path: &str) -> Result<bool, MyError> {
//!     match !path.ends_with(".md") {
//!         true => Err(MyError::MissingFilenameExtension),
//!         false => {
//!             let metadata = fs::metadata(path).map_err(|e| MyError::InternalError(InternalError::from_source(Box::new(e))))?;
//!             Ok(metadata.is_file())
//!         }
//!     }
//! }
//! ```

mod constraint_violation;
mod internal;
mod invalid_argument;
mod invalid_state;
mod unavailable;

pub use constraint_violation::{ConstraintViolationError, ConstraintViolationType};
pub use internal::InternalError;
pub use invalid_argument::InvalidArgumentError;
pub use invalid_state::InvalidStateError;
pub use unavailable::ResourceTemporarilyUnavailableError;

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum CliError {
    /// A subcommand requires one or more arguments, but none were provided.
    RequiresArgs,
    /// A non-existent subcommand was specified.
    InvalidSubcommand,
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
            // CliError::ClapError(err) => f.write_str(&err.message),
            CliError::ActionError(msg) => write!(f, "Subcommand encountered an error: {}", msg),
            CliError::EnvironmentError(msg) => f.write_str(msg),
        }
    }
}

// impl From<ClapError> for CliError {
//     fn from(err: ClapError) -> Self {
//         Self::ClapError(err)
//     }
// }
