//! # Secretary
//!
//! **Secretary** is a Rust library that transforms natural language into structured data using large language models (LLMs).
//! With its powerful derive macro system, you can extract structured information from unstructured text with minimal boilerplate code.
//!
//! ## Error Handling
//!
//! The library uses a unified error type, `SecretaryError`, to report issues.
//! A key variant is `SecretaryError::FieldDeserializationError`, which provides detailed context when the LLM's output
//! cannot be successfully parsed into your target struct. This error includes lists of both failed and successful fields,
//! making it easier to debug extraction failures, especially in distributed generation mode.

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
