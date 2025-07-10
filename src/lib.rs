pub mod constants;
pub mod error;
pub mod llm_providers;
pub mod message;
pub mod traits;

mod macros;
mod utilities;

// Re-export the main traits and derive macro for easy access
pub use traits::{AsyncGenerateData, GenerateData, IsLLM, Task};

// Re-export the derive macro
pub use secretary_derive::Task as TaskDerive;

// Re-export the errors
pub use error::SecretaryError;
