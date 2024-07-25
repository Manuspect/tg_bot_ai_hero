use std::error::Error;
use std::fmt;

#[cfg(any(feature = "postgres", feature = "sqlite"))]
use crate::modules::error::ConstraintViolationType;
use crate::modules::error::{
    ConstraintViolationError, InternalError, InvalidArgumentError, InvalidStateError,
};

/// Errors that may occur during [`MembersStore`](super::MembersStore) operations.
#[derive(Debug)]
pub enum MembersStoreError {
    ConstraintViolation(ConstraintViolationError),
    Internal(InternalError),
    InvalidArgument(InvalidArgumentError),
    InvalidState(InvalidStateError),
}

impl Error for MembersStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            MembersStoreError::ConstraintViolation(err) => err.source(),
            MembersStoreError::Internal(err) => err.source(),
            MembersStoreError::InvalidArgument(err) => err.source(),
            MembersStoreError::InvalidState(err) => err.source(),
        }
    }
}

impl fmt::Display for MembersStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MembersStoreError::ConstraintViolation(err) => f.write_str(&err.to_string()),
            MembersStoreError::Internal(err) => f.write_str(&err.to_string()),
            MembersStoreError::InvalidArgument(err) => f.write_str(&err.to_string()),
            MembersStoreError::InvalidState(err) => f.write_str(&err.to_string()),
        }
    }
}

#[cfg(feature = "diesel")]
impl From<diesel::r2d2::PoolError> for MembersStoreError {
    fn from(err: diesel::r2d2::PoolError) -> MembersStoreError {
        MembersStoreError::Internal(InternalError::with_message(err.to_string()))
    }
}

#[cfg(any(feature = "postgres", feature = "sqlite"))]
impl From<diesel::result::Error> for MembersStoreError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::DatabaseError(ref kind, _) => match kind {
                diesel::result::DatabaseErrorKind::UniqueViolation => {
                    MembersStoreError::ConstraintViolation(
                        ConstraintViolationError::from_source_with_violation_type(
                            ConstraintViolationType::Unique,
                            Box::new(err),
                        ),
                    )
                }
                diesel::result::DatabaseErrorKind::ForeignKeyViolation => {
                    MembersStoreError::ConstraintViolation(
                        ConstraintViolationError::from_source_with_violation_type(
                            ConstraintViolationType::ForeignKey,
                            Box::new(err),
                        ),
                    )
                }
                _ => MembersStoreError::Internal(InternalError::with_message(err.to_string())),
            },
            _ => MembersStoreError::Internal(InternalError::with_message(err.to_string())),
        }
    }
}

impl From<InternalError> for MembersStoreError {
    fn from(err: InternalError) -> Self {
        Self::Internal(err)
    }
}
