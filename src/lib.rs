pub mod llm_providers;
pub mod traits;
pub mod error;

// Re-export the main traits and derive macro for easy access
pub use traits::{AsyncGenerateData, FromJSON, GenerateData, IsLLM, Task, ToJSON};

// Re-export the derive macro
pub use secretary_derive::Task as TaskDerive;

// Re-export the errors
pub use error::SecretaryError;
