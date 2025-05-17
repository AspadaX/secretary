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
secretary = "0.1.12"
```

### Basic Example

```rust
use secretary::{openai::OpenAILLM, tasks::basic_task::BasicTask, traits::GenerateJSON};
use serde::{Deserialize, Serialize};

// Define your output schema
#[derive(Debug, Serialize, Deserialize)]
struct UserInfo {
    name: String,
    age: u8,
    interests: Vec<String>,
}

fn main() {
    // Initialize LLM client
    let llm = OpenAILLM::new(
        "https://api.openai.com/v1",
        &std::env::var("OPENAI_API_KEY").unwrap(),
        "gpt-4o-mini",
    ).unwrap();

    // Create a task with schema and instructions
    let task = BasicTask::new(
        UserInfo {
            name: "User's full name".to_string(),
            age: 0, // Placeholder for user's age
            interests: vec!["List of user's hobbies or interests".to_string()],
        },
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
2. **Create a Task**: Initialize a task with your schema and any additional instructions
3. **Process Text**: Send natural language input to an LLM through the Secretary API
4. **Get Structured Data**: Receive a JSON result that matches your defined schema

## Advanced Features

### Multi-Turn Conversations

Secretary supports contextual, multi-turn conversations that build data progressively:

```rust
// See the examples directory for complete multi-turn conversation examples
let mut task = BasicTask::new(schema, instructions);

// First user message
task.push(Role::User, "I'm planning a trip to Japan next spring");

// Generate response based on context
let response = llm.generate_json_with_context(&task).unwrap();

// Add response to conversation context
task.push(Role::Assistant, &response).unwrap();

// Continue conversation
task.push(Role::User, "I'll be there for about 10 days").unwrap();
```

### Contextual Tasks

For complex data gathering, use `ContextualTask` to maintain reasoning, notes, and follow-up questions:

```rust
let mut task = ContextualTask::new(schema, instructions);

// ContextualTask automatically manages reasoning, notes,
// and generates appropriate follow-up questions
```

## Documentation

- [Getting Started](./docs/GETTING_STARTED.md) - Complete setup guide
- [Examples](./docs/EXAMPLES.md) - Practical code examples
- [Project Structure](./docs/PROJECT_STRUCTURE.md) - Architecture overview
- [API Documentation](https://docs.rs/secretary) - Detailed API reference

## Environment Setup

Secretary requires the following environment variables:

```bash
export SECRETARY_OPENAI_API_BASE="https://api.openai.com/v1"
export SECRETARY_OPENAI_API_KEY="your-api-key"
export SECRETARY_OPENAI_MODEL="gpt-4o-mini"  # or other compatible model
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.