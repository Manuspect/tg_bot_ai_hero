use std::error::Error;
use std::fmt;

#[cfg(any(feature = "postgres", feature = "sqlite"))]
use crate::modules::error::ConstraintViolationType;
use crate::modules::error::{
    ConstraintViolationError, InternalError, InvalidArgumentError, InvalidStateError,
};

/// Errors that may occur during [`UserProfileStore`](super::UserProfileStore) operations.
#[derive(Debug)]
pub enum UserProfileStoreError {
    ConstraintViolation(ConstraintViolationError),
    Internal(InternalError),
    InvalidArgument(InvalidArgumentError),
    InvalidState(InvalidStateError),
}

impl Error for UserProfileStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            UserProfileStoreError::ConstraintViolation(err) => err.source(),
            UserProfileStoreError::Internal(err) => err.source(),
            UserProfileStoreError::InvalidArgument(err) => err.source(),
            UserProfileStoreError::InvalidState(err) => err.source(),
        }
    }
}

impl fmt::Display for UserProfileStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UserProfileStoreError::ConstraintViolation(err) => f.write_str(&err.to_string()),
            UserProfileStoreError::Internal(err) => f.write_str(&err.to_string()),
            UserProfileStoreError::InvalidArgument(err) => f.write_str(&err.to_string()),
            UserProfileStoreError::InvalidState(err) => f.write_str(&err.to_string()),
        }
    }
}

#[cfg(feature = "diesel")]
impl From<diesel::r2d2::PoolError> for UserProfileStoreError {
    fn from(err: diesel::r2d2::PoolError) -> UserProfileStoreError {
        UserProfileStoreError::Internal(InternalError::with_message(err.to_string()))
    }
}

#[cfg(any(feature = "postgres", feature = "sqlite"))]
impl From<diesel::result::Error> for UserProfileStoreError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::DatabaseError(ref kind, _) => match kind {
                diesel::result::DatabaseErrorKind::UniqueViolation => {
                    UserProfileStoreError::ConstraintViolation(
                        ConstraintViolationError::from_source_with_violation_type(
                            ConstraintViolationType::Unique,
                            Box::new(err),
                        ),
                    )
                }
                diesel::result::DatabaseErrorKind::ForeignKeyViolation => {
                    UserProfileStoreError::ConstraintViolation(
                        ConstraintViolationError::from_source_with_violation_type(
                            ConstraintViolationType::ForeignKey,
                            Box::new(err),
                        ),
                    )
                }
                _ => UserProfileStoreError::Internal(InternalError::with_message(err.to_string())),
            },
            _ => UserProfileStoreError::Internal(InternalError::with_message(err.to_string())),
        }
    }
}

impl From<InternalError> for UserProfileStoreError {
    fn from(err: InternalError) -> Self {
        Self::Internal(err)
    }
}
