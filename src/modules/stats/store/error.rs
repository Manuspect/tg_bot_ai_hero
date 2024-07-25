use std::error::Error;
use std::fmt;

#[cfg(any(feature = "postgres", feature = "sqlite"))]
use crate::modules::error::ConstraintViolationType;
use crate::modules::error::{
    ConstraintViolationError, InternalError, InvalidArgumentError, InvalidStateError,
};

/// Errors that may occur during [`TokenUsageStore`](super::TokenUsageStore) operations.
#[derive(Debug)]
pub enum TokenUsageStoreError {
    ConstraintViolation(ConstraintViolationError),
    Internal(InternalError),
    InvalidArgument(InvalidArgumentError),
    InvalidState(InvalidStateError),
}

impl Error for TokenUsageStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            TokenUsageStoreError::ConstraintViolation(err) => err.source(),
            TokenUsageStoreError::Internal(err) => err.source(),
            TokenUsageStoreError::InvalidArgument(err) => err.source(),
            TokenUsageStoreError::InvalidState(err) => err.source(),
        }
    }
}

impl fmt::Display for TokenUsageStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenUsageStoreError::ConstraintViolation(err) => f.write_str(&err.to_string()),
            TokenUsageStoreError::Internal(err) => f.write_str(&err.to_string()),
            TokenUsageStoreError::InvalidArgument(err) => f.write_str(&err.to_string()),
            TokenUsageStoreError::InvalidState(err) => f.write_str(&err.to_string()),
        }
    }
}

#[cfg(feature = "diesel")]
impl From<diesel::r2d2::PoolError> for TokenUsageStoreError {
    fn from(err: diesel::r2d2::PoolError) -> TokenUsageStoreError {
        TokenUsageStoreError::Internal(InternalError::with_message(err.to_string()))
    }
}

#[cfg(any(feature = "postgres", feature = "sqlite"))]
impl From<diesel::result::Error> for TokenUsageStoreError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::DatabaseError(ref kind, _) => match kind {
                diesel::result::DatabaseErrorKind::UniqueViolation => {
                    TokenUsageStoreError::ConstraintViolation(
                        ConstraintViolationError::from_source_with_violation_type(
                            ConstraintViolationType::Unique,
                            Box::new(err),
                        ),
                    )
                }
                diesel::result::DatabaseErrorKind::ForeignKeyViolation => {
                    TokenUsageStoreError::ConstraintViolation(
                        ConstraintViolationError::from_source_with_violation_type(
                            ConstraintViolationType::ForeignKey,
                            Box::new(err),
                        ),
                    )
                }
                _ => TokenUsageStoreError::Internal(InternalError::with_message(err.to_string())),
            },
            _ => TokenUsageStoreError::Internal(InternalError::with_message(err.to_string())),
        }
    }
}

impl From<InternalError> for TokenUsageStoreError {
    fn from(err: InternalError) -> Self {
        Self::Internal(err)
    }
}
