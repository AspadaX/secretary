# Secretary

[![Crates.io](https://img.shields.io/crates/v/secretary.svg)](https://crates.io/crates/secretary)
[![API Docs](https://docs.rs/secretary/badge.svg)](https://docs.rs/secretary)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Secretary** is a Rust library that transforms natural language into structured data using large language models (LLMs). With its powerful derive macro system, you can extract structured information from unstructured text with minimal boilerplate code.

## Features

- 🚀 **Unified Task Trait**: Single trait combining data extraction, schema definition, and system prompt generation with `#[derive(Task)]`
- 🔍 **Schema-Based Extraction**: Define your data structure using Rust structs with field-level instructions
- 🔄 **Context-Aware Conversations**: Maintain conversation state for multi-turn interactions
- 📋 **Declarative Field Instructions**: Use `#[task(instruction = "...")]` attributes to guide extraction
- ⚡ **Async Support**: Built-in async/await support for concurrent processing
- 🔌 **Extensible LLM Support**: Currently supports OpenAI API with more providers planned
- 🛡️ **Type Safety**: Leverage Rust's type system for reliable data extraction
- 🧹 **Simplified API**: Consolidated traits reduce boilerplate and complexity

## Quick Start

```bash
cargo add secretary
```

### Basic Example

```rust
use secretary::Task;
use secretary::llm_providers::openai::OpenAILLM;
use secretary::traits::GenerateJSON;
use serde::{Serialize, Deserialize};

// Define your data structure with extraction instructions
#[derive(Task, Serialize, Deserialize, Debug, Default)]
struct PersonInfo {
    // Required fields for Task trait
    #[serde(skip)]
    pub context: secretary::MessageList,
    #[serde(skip)]
    pub additional_instructions: Vec<String>,
    
    // Data fields with specific extraction instructions
    #[task(instruction = "Extract the person's full name")]
    pub name: String,
    
    #[task(instruction = "Extract age as a number")]
    pub age: u32,
    
    #[task(instruction = "Extract email address if mentioned")]
    pub email: Option<String>,
    
    #[task(instruction = "List all hobbies or interests mentioned")]
    pub interests: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    // Create a task instance with additional instructions
    let task = PersonInfo::new(vec![
        "Be precise with personal information".to_string(),
        "Use 'Unknown' for missing data".to_string(),
    ]);
    
    // Initialize LLM client
    let llm = OpenAILLM::new(
        "https://api.openai.com/v1",
        "your-api-key",
        "gpt-4"
    )?;
    
    // Process natural language input
    let input = "Hi, I'm Jane Smith, 29 years old. My email is jane@example.com. I love hiking, coding, and playing piano.";
    let json_result = llm.generate_json(&task, input)?;
    
    // Parse result back to struct
    let person: PersonInfo = serde_json::from_str(&json_result)?;
    println!("{:#?}", person);
    
    Ok(())
}
```

## How It Works

1. **Define Your Schema**: Create a Rust struct with `#[derive(Task)]` and field-level instructions
2. **Add Required Fields**: Include `context` and `additional_instructions` fields (marked with `#[serde(skip)]`)
3. **Annotate Fields**: Use `#[task(instruction = "...")]` to guide the LLM on how to extract each field
4. **Automatic Implementation**: The derive macro implements all necessary traits (data model, system prompt generation, context management)
5. **Create Task Instance**: Initialize with `YourStruct::new(additional_instructions)`
6. **Process Text**: Send natural language input to an LLM through the Secretary API
7. **Get Structured Data**: Receive JSON that can be parsed back into your struct

### Field Instructions

The `#[task(instruction = "...")]` attribute tells the LLM how to extract each field:

```rust
#[derive(Task, Serialize, Deserialize, Debug, Default)]
struct ProductInfo {
    #[serde(skip)]
    pub context: secretary::MessageList,
    #[serde(skip)]
    pub additional_instructions: Vec<String>,
    
    #[task(instruction = "Extract the product name or title")]
    pub name: String,
    
    #[task(instruction = "Extract price as a number without currency symbols")]
    pub price: f64,
    
    #[task(instruction = "Categorize the product type (electronics, clothing, etc.)")]
    pub category: String,
    
    #[task(instruction = "Extract brand name if mentioned, otherwise null")]
    pub brand: Option<String>,
    
    #[task(instruction = "Determine if product is available (true/false)")]
    pub in_stock: bool,
}
```

## Advanced Features

### Async Processing

Secretary provides full async support for concurrent processing:

```rust
use secretary::traits::AsyncGenerateJSON;
use tokio;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let llm = OpenAILLM::new("https://api.openai.com/v1", "your-api-key", "gpt-4")?;
    let task = PersonInfo::new(vec!["Extract accurately".to_string()]);
    
    // Process multiple inputs concurrently
    let inputs = vec![
        "John Doe, 25, loves gaming",
        "Alice Smith, 30, enjoys reading and cooking",
        "Bob Johnson, 35, passionate about photography",
    ];
    
    let futures: Vec<_> = inputs.into_iter().map(|input| {
        let llm = &llm;
        let task = &task;
        async move {
            llm.async_generate_json(task, input).await
        }
    }).collect();
    
    let results = futures::future::join_all(futures).await;
    
    for result in results {
        match result {
            Ok(json) => println!("Extracted: {}", json),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    
    Ok(())
}
```

### Context-Aware Conversations

Maintain conversation state for multi-turn interactions:

```rust
use secretary::message_list::Role;

fn main() -> anyhow::Result<()> {
    let mut task = PersonInfo::new(vec!["Gather information progressively".to_string()]);
    let llm = OpenAILLM::new("https://api.openai.com/v1", "your-api-key", "gpt-4")?;
    
    // First interaction
    task.push(Role::User, "Hi, I'm John")?;
    let response1 = llm.generate_json(&task, "")?;
    task.push(Role::Assistant, &response1)?;
    
    // Continue conversation with context
    task.push(Role::User, "I'm 25 years old and love programming")?;
    let response2 = llm.generate_json(&task, "")?;
    
    println!("Final result: {}", response2);
    Ok(())
}
```

### System Prompt Generation

The derive macro automatically generates comprehensive system prompts:

```rust
let task = PersonInfo::new(vec!["Be accurate".to_string()]);
let prompt = task.get_system_prompt();
println!("{}", prompt);

// Output includes:
// - JSON structure specification
// - Field-specific instructions
// - Additional instructions
// - Formatting guidelines
```

## Examples

The `examples/` directory contains practical demonstrations:

### Basic Usage
- **`derive_example.rs`** - Basic person information extraction
- **`async_example.rs`** - Async product information extraction with comprehensive testing

Run examples with:
```bash
# Basic example (no API key required for demo)
cargo run --example derive_example

# Async example (no API key required for demo)
cargo run --example async_example

# To test with real API (uncomment API calls in examples):
export OPENAI_API_KEY="your-api-key"
cargo run --example async_example
```

## Environment Setup

For production use with OpenAI:

```bash
export OPENAI_API_KEY="your-openai-api-key"
```

In your code:
```rust
let api_key = std::env::var("OPENAI_API_KEY")
    .expect("OPENAI_API_KEY environment variable not set");

let llm = OpenAILLM::new(
    "https://api.openai.com/v1",
    &api_key,
    "gpt-4"
)?;
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
