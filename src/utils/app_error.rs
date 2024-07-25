// This file contains a custom error for the application.

/// The structure that represents a custom error.
#[derive(Debug)]
pub struct AppError {
    // Error message.
    pub message: String,
} // end struct AppError

// Implement functionality.
impl AppError {
    /// This function creates an instances of the custom error.
    pub fn new(message: impl AsRef<str>) -> Self {
        Self {
            message: message.as_ref().to_string(),
        } // end Self
    } // end fn new
} // end impl AppError

/// Implement an opportunity for an error to be displayed.
impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Simply write the message field to the formatter
        write!(f, "{}", self.message)
    } // end fn fmt
} // end impl std::fmt::Display for AppError
