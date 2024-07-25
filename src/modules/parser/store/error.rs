use std::error::Error;
use std::fmt;

#[cfg(any(feature = "postgres", feature = "sqlite"))]
use crate::modules::error::ConstraintViolationType;
use crate::modules::error::{
    ConstraintViolationError, InternalError, InvalidArgumentError, InvalidStateError,
};

/// Errors that may occur during [`ChannelMessagesStore`](super::ChannelMessagesStore) operations.
#[derive(Debug)]
pub enum ChannelMessagesStoreError {
    ConstraintViolation(ConstraintViolationError),
    Internal(InternalError),
    InvalidArgument(InvalidArgumentError),
    InvalidState(InvalidStateError),
}

impl Error for ChannelMessagesStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ChannelMessagesStoreError::ConstraintViolation(err) => err.source(),
            ChannelMessagesStoreError::Internal(err) => err.source(),
            ChannelMessagesStoreError::InvalidArgument(err) => err.source(),
            ChannelMessagesStoreError::InvalidState(err) => err.source(),
        }
    }
}

impl fmt::Display for ChannelMessagesStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ChannelMessagesStoreError::ConstraintViolation(err) => f.write_str(&err.to_string()),
            ChannelMessagesStoreError::Internal(err) => f.write_str(&err.to_string()),
            ChannelMessagesStoreError::InvalidArgument(err) => f.write_str(&err.to_string()),
            ChannelMessagesStoreError::InvalidState(err) => f.write_str(&err.to_string()),
        }
    }
}

#[cfg(feature = "diesel")]
impl From<diesel::r2d2::PoolError> for ChannelMessagesStoreError {
    fn from(err: diesel::r2d2::PoolError) -> ChannelMessagesStoreError {
        ChannelMessagesStoreError::Internal(InternalError::with_message(err.to_string()))
    }
}

#[cfg(any(feature = "postgres", feature = "sqlite"))]
impl From<diesel::result::Error> for ChannelMessagesStoreError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::DatabaseError(ref kind, _) => match kind {
                diesel::result::DatabaseErrorKind::UniqueViolation => {
                    ChannelMessagesStoreError::ConstraintViolation(
                        ConstraintViolationError::from_source_with_violation_type(
                            ConstraintViolationType::Unique,
                            Box::new(err),
                        ),
                    )
                }
                diesel::result::DatabaseErrorKind::ForeignKeyViolation => {
                    ChannelMessagesStoreError::ConstraintViolation(
                        ConstraintViolationError::from_source_with_violation_type(
                            ConstraintViolationType::ForeignKey,
                            Box::new(err),
                        ),
                    )
                }
                _ => ChannelMessagesStoreError::Internal(InternalError::with_message(
                    err.to_string(),
                )),
            },
            _ => ChannelMessagesStoreError::Internal(InternalError::with_message(err.to_string())),
        }
    }
}

impl From<InternalError> for ChannelMessagesStoreError {
    fn from(err: InternalError) -> Self {
        Self::Internal(err)
    }
}
