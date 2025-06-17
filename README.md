# Secretary 🚀

[![Crates.io](https://img.shields.io/crates/v/secretary.svg)](https://crates.io/crates/secretary)
[![API Docs](https://docs.rs/secretary/badge.svg)](https://docs.rs/secretary)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Secretary** is a Rust library that translates natural language into structured data using large language models (LLMs). It provides a simple, type-safe way to extract structured information from unstructured text.

## Features

- 🔍 **Schema-Based Extraction**: Define your data structure using Rust structs and let LLMs extract matching data
- 🔄 **Context-Aware Conversations**: Maintain conversation state for multi-turn interactions
- 🧠 **Progressive Data Building**: Incrementally build complex data structures through conversational interactions
- 📋 **Declarative Schema Annotations**: Document your schemas with field descriptions that guide the LLM
- 🔌 **Extensible LLM Support**: Currently supports OpenAI API with more providers planned

## Quick Start

```bash
cargo add secretary
```

Or, 

```toml
[dependencies]
secretary = "0.2.30"
```

### Basic Example

```rust
use secretary::{
    llm_providers::openai::OpenAILLM, 
    tasks::basic_task::BasicTask, 
    traits::{DataModel, GenerateJSON}
};
use serde::{Deserialize, Serialize};

// Define your output schema
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserInfo {
    name: String,
    age: u8,
    interests: Vec<String>,
}

// Implement DataModel to provide instructions for the schema
impl DataModel for UserInfo {
    fn provide_data_model_instructions() -> Self {
        Self {
            name: "User's full name".to_string(),
            age: 0, // Age in years
            interests: vec!["List of user's hobbies or interests".to_string()],
        }
    }
}

fn main() {
    // Initialize LLM client with environment variables
    let llm = OpenAILLM::new(
        &std::env::var("SECRETARY_OPENAI_API_BASE").unwrap(),
        &std::env::var("SECRETARY_OPENAI_API_KEY").unwrap(),
        &std::env::var("SECRETARY_OPENAI_MODEL").unwrap(),
    ).unwrap();

    // Create a task with schema and additional instructions
    let task = BasicTask::new::<UserInfo>(
        vec![
            "Extract the user's personal information from the text.".to_string(),
            "Include all interests mentioned in the text.".to_string(),
        ],
    );

    // Process natural language input
    let input = "Hi, I'm Jane Smith, 29 years old. I love hiking, coding, and playing piano.";
    let json_result = llm.generate_json(&task, input).unwrap();
    
    println!("{}", json_result);
    // Output: {"name":"Jane Smith","age":29,"interests":["hiking","coding","playing piano"]}
}
```

## How It Works

1. **Define Your Schema**: Create a Rust struct that represents the data structure you want to extract
2. **Implement DataModel**: Provide instructions for each field using the `DataModel` trait
3. **Create a Task**: Initialize a task with your schema and any additional instructions
4. **Process Text**: Send natural language input to an LLM through the Secretary API
5. **Get Structured Data**: Receive a JSON result that matches your defined schema

### The DataModel Trait

The `DataModel` trait is essential for guiding the LLM on how to populate your schema:

```rust
use secretary::traits::DataModel;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProductInfo {
    name: String,
    category: String,
    price_range: Option<String>,
}

impl DataModel for ProductInfo {
    fn provide_data_model_instructions() -> Self {
        Self {
            name: "The product name as mentioned in the text".to_string(),
            category: "Product category (electronics, clothing, etc.)".to_string(),
            price_range: Some("Price range if mentioned (e.g., '$10-20')".to_string()),
        }
    }
}
```

## Advanced Features

### Multi-Turn Conversations

Secretary supports contextual, multi-turn conversations that build data progressively:

```rust
use secretary::{
    llm_providers::openai::OpenAILLM,
    tasks::basic_task::BasicTask,
    traits::{Context, DataModel, GenerateJSON},
    message_list::Role,
};

// Initialize your schema and task
let mut task = BasicTask::new::<YourSchema>(instructions);

// First user message
task.push(Role::User, "I'm planning a trip to Japan next spring").unwrap();

// Generate response based on context
let response = llm.generate_json_with_context(&task).unwrap();

// Add response to conversation context
task.push(Role::Assistant, &response).unwrap();

// Continue conversation
task.push(Role::User, "I'll be there for about 10 days").unwrap();
let response2 = llm.generate_json_with_context(&task).unwrap();
```

### Contextual Tasks with Reasoning

For complex data gathering, use `ContextualTask` to maintain reasoning, notes, and follow-up questions:

```rust
use secretary::{
    llm_providers::openai::OpenAILLM,
    tasks::contextual_task::ContextualTask,
    traits::{Context, DataModel, GenerateJSON},
};

// Create a contextual task for complex interactions
let mut task = ContextualTask::new::<YourSchema>(
    vec![
        "Gather information progressively through conversation.".to_string(),
        "Ask follow-up questions when information is incomplete.".to_string(),
    ],
);

// Process user input
task.push(Role::User, "I need help planning something").unwrap();
let response = llm.generate_json_with_context(&task).unwrap();

// The response includes reasoning, notes, and structured data
// ContextualTask automatically manages reasoning and follow-up questions
```

### Async Processing

Secretary supports async operations for concurrent processing:

```rust
use secretary::{
    llm_providers::openai::OpenAILLM,
    tasks::basic_task::BasicTask,
    traits::{AsyncGenerateJSON, DataModel},
};

#[tokio::main]
async fn main() {
    let llm = OpenAILLM::new(/* ... */).unwrap();
    let task = BasicTask::new::<YourSchema>(instructions);
    
    // Process multiple inputs concurrently
    let futures = inputs.into_iter().map(|input| {
        let llm = Arc::clone(&llm);
        let task = task.clone();
        tokio::spawn(async move {
            llm.async_generate_json(&task, &input).await
        })
    });
    
    let results = futures::future::join_all(futures).await;
}
```

## Examples

The `examples/` directory contains comprehensive examples demonstrating different use cases:

### Basic Usage
- **`generate_json.rs`** - Simple sentiment analysis with structured output
- **`generate_json_with_context.rs`** - Multi-turn conversation with context preservation

### Async Processing
- **`async_generate_json.rs`** - Concurrent processing of multiple requests
- **`async_generate_json_with_context.rs`** - Async multi-turn conversations

### Contextual Tasks
- **`contextual_prompt_basic.rs`** - Product analysis with reasoning and notes
- **`contextual_prompt_conversation.rs`** - Interactive trip planning conversation
- **`contextual_prompt_analysis.rs`** - Advanced contextual analysis patterns

### Real-World Applications
- **`product_review_analysis.rs`** - E-commerce review processing

Run any example with:
```bash
# Set environment variables first
export SECRETARY_OPENAI_API_BASE="https://api.openai.com/v1"
export SECRETARY_OPENAI_API_KEY="your-api-key"
export SECRETARY_OPENAI_MODEL="gpt-4o-mini"

# Run an example
cargo run --example generate_json
```

## Documentation

- [Getting Started](./docs/GETTING_STARTED.md) - Complete setup guide
- [Examples](./docs/EXAMPLES.md) - Practical code examples
- [Project Structure](./docs/PROJECT_STRUCTURE.md) - Architecture overview
- [API Documentation](https://docs.rs/secretary) - Detailed API reference

## Environment Setup

Secretary requires the following environment variables for OpenAI integration:

```bash
export SECRETARY_OPENAI_API_BASE="https://api.openai.com/v1"
export SECRETARY_OPENAI_API_KEY="your-openai-api-key"
export SECRETARY_OPENAI_MODEL="gpt-4o-mini"  # or gpt-4o, gpt-3.5-turbo, etc.
```

These environment variables are used by the examples and can be referenced in your code as:
```rust
let llm = OpenAILLM::new(
    &std::env::var("SECRETARY_OPENAI_API_BASE").unwrap(),
    &std::env::var("SECRETARY_OPENAI_API_KEY").unwrap(),
    &std::env::var("SECRETARY_OPENAI_MODEL").unwrap(),
).unwrap();
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
