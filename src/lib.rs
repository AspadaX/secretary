pub mod llm_providers;
pub mod traits;

// Re-export the main traits and derive macro for easy access
pub use traits::{AsyncGenerateData, FromJSON, GenerateData, IsLLM, Task, ToJSON};

// Re-export the derive macro
pub use secretary_derive::Task as TaskDerive;
