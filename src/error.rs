/// Custom error type for secretary
#[derive(Debug)]
pub enum SecretaryError {
    TokioRuntime(std::io::Error),
    SerdeJsonError(serde_json::Error),
    NoLLMResponse,
    BuildRequestError(String),
}

impl std::fmt::Display for SecretaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecretaryError::TokioRuntime(e) => write!(f, "Tokio runtime error: {}", e),
            SecretaryError::SerdeJsonError(e) => write!(f, "Serde JSON error: {}", e),
            SecretaryError::NoLLMResponse => write!(f, "No response is retrieved from the LLM"),
            SecretaryError::BuildRequestError(e) => write!(f, "Failed to build request: {}", e),
        }
    }
}

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
