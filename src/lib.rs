pub mod llm_providers;
pub mod message_list;
pub mod traits;

// Re-export the main traits and derive macro for easy access
pub use message_list::{Message, MessageList, Role};
pub use traits::{AsyncGenerateData, Context, FromJSON, GenerateData, IsLLM, Task, ToJSON};

// Re-export the derive macro
pub use secretary_derive::Task as TaskDerive;
