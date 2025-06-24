pub mod message_list;
pub mod traits;
pub mod llm_providers;

// Re-export the main traits and derive macro for easy access
pub use traits::{Task, GenerateData, AsyncGenerateData, IsLLM, ToJSON, FromJSON};
pub use message_list::{Message, MessageList, Role};
