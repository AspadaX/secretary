/// Custom error type for the `secretary` library.
///
/// This enum consolidates all possible errors that can occur during the data extraction process,
/// providing a unified interface for error handling.
#[derive(Debug)]
pub enum SecretaryError {
    TokioRuntime(std::io::Error),
    SerdeJsonError(serde_json::Error),
    JsonParsingError(String),
    NoLLMResponse,
    BuildRequestError(String),
    /// Indicates a failure during the deserialization of individual fields from the LLM's response.
    ///
    /// This error is particularly useful for debugging issues with distributed generation,
    /// as it provides detailed information about which fields were successfully parsed and which failed.
    FieldDeserializationError(FieldDeserializationError),
}

/// A detailed error report for field-level deserialization failures.
///
/// This struct is returned when `generate_from_tuples!` fails to construct the target struct
/// from the key-value pairs returned by the LLM. It captures which fields succeeded, which failed,
/// and the underlying deserialization error.
#[derive(Debug, Clone)]
pub struct FieldDeserializationError {
    /// A list of field names that could not be successfully deserialized.
    pub failed_fields: Vec<String>,
    /// A list of field names that were successfully deserialized before the error occurred.
    pub successful_fields: Vec<String>,
    /// The original `serde_json::Error` message that caused the failure, converted to a string.
    pub original_error: String,
}

impl std::fmt::Display for SecretaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecretaryError::TokioRuntime(e) => write!(f, "Tokio runtime error: {}", e),
            SecretaryError::SerdeJsonError(e) => write!(f, "Serde JSON error: {}", e),
            SecretaryError::NoLLMResponse => write!(f, "No response is retrieved from the LLM"),
            SecretaryError::BuildRequestError(e) => write!(f, "Failed to build request: {}", e),
            SecretaryError::JsonParsingError(e) => {
                write!(f, "LLM generated a malformed json. Error message: {}", e)
            }
            SecretaryError::FieldDeserializationError(e) => {
                write!(f, "Field deserialization failed: {}", e)
            }
        }
    }
}

impl std::fmt::Display for FieldDeserializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to deserialize {} field(s): [{}]. Successfully parsed {} field(s): [{}]. Original error: {}",
            self.failed_fields.len(),
            self.failed_fields.join(", "),
            self.successful_fields.len(),
            self.successful_fields.join(", "),
            self.original_error
        )
    }
}

impl std::error::Error for FieldDeserializationError {}

impl std::error::Error for SecretaryError {}

impl From<serde_json::Error> for SecretaryError {
    fn from(e: serde_json::Error) -> Self {
        SecretaryError::SerdeJsonError(e)
    }
}

impl From<std::io::Error> for SecretaryError {
    fn from(e: std::io::Error) -> Self {
        SecretaryError::TokioRuntime(e)
    }
}

impl From<FieldDeserializationError> for SecretaryError {
    fn from(e: FieldDeserializationError) -> Self {
        SecretaryError::FieldDeserializationError(e)
    }
}
