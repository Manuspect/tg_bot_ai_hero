use std::error::Error;
use std::fmt;

#[cfg(any(feature = "postgres", feature = "sqlite"))]
use crate::modules::error::ConstraintViolationType;
use crate::modules::error::{
    ConstraintViolationError, InternalError, InvalidArgumentError, InvalidStateError,
};

/// Errors that may occur during [`PreferencesStore`](super::PreferencesStore) operations.
#[derive(Debug)]
pub enum PreferencesStoreError {
    ConstraintViolation(ConstraintViolationError),
    Internal(InternalError),
    InvalidArgument(InvalidArgumentError),
    InvalidState(InvalidStateError),
}

impl Error for PreferencesStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            PreferencesStoreError::ConstraintViolation(err) => err.source(),
            PreferencesStoreError::Internal(err) => err.source(),
            PreferencesStoreError::InvalidArgument(err) => err.source(),
            PreferencesStoreError::InvalidState(err) => err.source(),
        }
    }
}

impl fmt::Display for PreferencesStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PreferencesStoreError::ConstraintViolation(err) => f.write_str(&err.to_string()),
            PreferencesStoreError::Internal(err) => f.write_str(&err.to_string()),
            PreferencesStoreError::InvalidArgument(err) => f.write_str(&err.to_string()),
            PreferencesStoreError::InvalidState(err) => f.write_str(&err.to_string()),
        }
    }
}

#[cfg(feature = "diesel")]
impl From<diesel::r2d2::PoolError> for PreferencesStoreError {
    fn from(err: diesel::r2d2::PoolError) -> PreferencesStoreError {
        PreferencesStoreError::Internal(InternalError::from_source(Box::new(err)))
    }
}

#[cfg(any(feature = "postgres", feature = "sqlite"))]
impl From<diesel::result::Error> for PreferencesStoreError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::DatabaseError(ref kind, _) => match kind {
                diesel::result::DatabaseErrorKind::UniqueViolation => {
                    PreferencesStoreError::ConstraintViolation(
                        ConstraintViolationError::from_source_with_violation_type(
                            ConstraintViolationType::Unique,
                            Box::new(err),
                        ),
                    )
                }
                diesel::result::DatabaseErrorKind::ForeignKeyViolation => {
                    PreferencesStoreError::ConstraintViolation(
                        ConstraintViolationError::from_source_with_violation_type(
                            ConstraintViolationType::ForeignKey,
                            Box::new(err),
                        ),
                    )
                }
                _ => PreferencesStoreError::Internal(InternalError::from_source(Box::new(err))),
            },
            _ => PreferencesStoreError::Internal(InternalError::from_source(Box::new(err))),
        }
    }
}

impl From<InternalError> for PreferencesStoreError {
    fn from(err: InternalError) -> Self {
        Self::Internal(err)
    }
}
