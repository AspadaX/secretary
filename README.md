# Secretary

[![Crates.io](https://img.shields.io/crates/v/secretary.svg)](https://crates.io/crates/secretary)
[![API Docs](https://docs.rs/secretary/badge.svg)](https://docs.rs/secretary)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Secretary** is a Rust library that transforms natural language into structured data using large language models (LLMs). With its powerful derive macro system, you can extract structured information from unstructured text with minimal boilerplate code.

## Table of Contents

- [Secretary](#secretary)
  - [Table of Contents](#table-of-contents)
    - [Basic Example](#basic-example)
  - [How It Works](#how-it-works)
    - [Field Instructions](#field-instructions)
  - [Advanced Features](#advanced-features)
    - [Async Processing](#async-processing)
    - [Distributed Field-Level Generation](#distributed-field-level-generation)
    - [Multiple Extractions](#multiple-extractions)
    - [Force Generation for Models Without a JSON Mode](#force-generation-for-models-without-a-json-mode)
    - [System Prompt Generation](#system-prompt-generation)
  - [Examples](#examples)
    - [Basic Usage](#basic-usage)
    - [Distributed Generation](#distributed-generation)
    - [Force Generation (for Reasoning Models)](#force-generation-for-reasoning-models)
  - [LLM Provider Setup](#llm-provider-setup)
    - [OpenAI](#openai)
    - [Azure OpenAI](#azure-openai)
  - [API Reference](#api-reference)
    - [Core Traits](#core-traits)
    - [LLM Providers](#llm-providers)
    - [Derive Macro (secretary-derive)](#derive-macro-secretary-derive)
  - [Error Handling](#error-handling)
    - [`FieldDeserializationError`](#fielddeserializationerror)
  - [Troubleshooting](#troubleshooting)
    - [Common Issues](#common-issues)
    - [Performance Tips](#performance-tips)
  - [Roadmap](#roadmap)
    - [Dependencies](#dependencies)
  - [Contributing](#contributing)
  - [License](#license)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

## Features

- 🚀 **Unified Task Trait**: Single trait combining data extraction, schema definition, and system prompt generation with `#[derive(Task)]`
- 🔍 **Schema-Based Extraction**: Define your data structure using Rust structs with field-level instructions
- 📋 **Declarative Field Instructions**: Use `#[task(instruction = "...")]` attributes to guide extraction
- ⚡ **Async Support**: Built-in async/await support for concurrent processing
- 🎯 **Distributed Generation**: Field-level extraction for improved accuracy and error isolation
- 🧠 **Reasoning Model Support**: Force generation methods for models without JSON mode (o1, deepseek, etc.)
- 🔌 **Multiple LLM Providers**: Supports OpenAI API and Azure OpenAI with extensible provider system
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
use secretary::traits::GenerateData;
use serde::{Serialize, Deserialize};

// Define your data structure with extraction instructions
#[derive(Task, Serialize, Deserialize, Debug)]
struct PersonInfo {
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
    // Create a task instance
    let task = PersonInfo::new();
    
    // Additional instructions for the LLM
    let additional_instructions = vec![
        "Be precise with personal information".to_string(),
        "Use 'Unknown' for missing data".to_string(),
    ];
    
    // Initialize LLM client
    let llm = OpenAILLM::new(
        "https://api.openai.com/v1",
        "your-api-key",
        "gpt-4"
    )?;
    
    // Process natural language input
    let input = "Hi, I'm Jane Smith, 29 years old. My email is jane@example.com. I love hiking, coding, and playing piano.";
    
    // Process natural language input and get structured data directly
    let person: PersonInfo = llm.generate_data(&task, input, &additional_instructions)?;
    println!("{:#?}", person);
    
    Ok(())
}
```

## How It Works

1. **Define Your Schema**: Create a Rust struct with `#[derive(Task)]` and field-level instructions
2. **Annotate Fields**: Use `#[task(instruction = "...")]` to guide the LLM on how to extract each field
4. **Create Task Instance**: Initialize with `YourStruct::new()`
5. **Process Text**: Send natural language input to an LLM through the Secretary API with additional instructions
6. **Get Structured Data**: Receive structured data parsed into your struct

### Field Instructions

The `#[task(instruction = "...")]` attribute tells the LLM how to extract each field:

```rust
#[derive(Task, Serialize, Deserialize, Debug)]
struct ProductInfo {
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
use secretary::traits::AsyncGenerateData;
use tokio;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let llm = OpenAILLM::new("https://api.openai.com/v1", "your-api-key", "gpt-4")?;
    let task = PersonInfo::new();
    let additional_instructions = vec!["Extract accurately".to_string()];
    
    // Process multiple inputs concurrently
    let inputs = vec![
        "John Doe, 25, loves gaming",
        "Alice Smith, 30, enjoys reading and cooking",
        "Bob Johnson, 35, passionate about photography",
    ];
    
    let futures: Vec<_> = inputs.into_iter().map(|input| {
        let llm = &llm;
        let task = &task;
        let additional_instructions = &additional_instructions;
        async move {
            llm.async_generate_data(task, input, additional_instructions).await
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
### Distributed Field-Level Generation

For improved accuracy and better error isolation, Secretary supports distributed generation where each field is extracted separately and then combined. This approach is more resilient to failures in individual fields. If a field fails to deserialize, the system will now raise a `FieldDeserializationError`, pinpointing the exact issue without affecting the successfully extracted fields.
```rust
use secretary::traits::{GenerateData, AsyncGenerateData};

// Synchronous distributed generation
let result: PersonInfo = llm.fields_generate_data(&task, input, &additional_instructions)?;

// Asynchronous distributed generation
let result: PersonInfo = llm.async_fields_generate_data(&task, input, &additional_instructions).await?;
```

**Benefits of Distributed Generation:**
- **Improved accuracy**: Each field gets focused attention from the LLM
- **Parallel processing**: Multiple fields extracted simultaneously
- **Better for complex extractions**: Handles complex data structures more reliably

### Multiple Extractions

Process multiple inputs with the same task configuration:

```rust
fn main() -> anyhow::Result<()> {
    let task = PersonInfo::new();
    let additional_instructions = vec!["Extract all available information".to_string()];
    let llm = OpenAILLM::new("https://api.openai.com/v1", "your-api-key", "gpt-4")?;
    
    let inputs = vec![
        "Hi, I'm John, 25 years old",
        "Sarah works as a designer and is 30",
        "Mike's email is mike@example.com"
    ];
    
    for input in inputs {
        let person: PersonInfo = llm.generate_data(&task, input, &additional_instructions)?;
        println!("{:#?}", person);
    }
    
    Ok(())
}
```

### Force Generation for Models Without a JSON Mode

Secretary supports reasoning models like o1 and deepseek that don't have built-in JSON mode support through force generation methods:

```rust
use secretary::traits::{GenerateData, AsyncGenerateData};

// Synchronous force generation
let result: PersonInfo = llm.force_generate_data(&task, input, &additional_instructions)?;

// Asynchronous force generation
let result: PersonInfo = llm.async_force_generate_data(&task, input, &additional_instructions).await?;
```

### System Prompt Generation

The derive macro automatically generates comprehensive system prompts:

```rust
let task = PersonInfo::new();
let prompt = task.get_system_prompt();
println!("{}", prompt);

// Output includes:
// - JSON structure specification
// - Field-specific extraction instructions
// - Response format requirements
```

## Examples

The `examples/` directory contains practical demonstrations:

### Basic Usage
- **`sync.rs`** - Basic person information extraction using synchronous API
- **`async.rs`** - Async product information extraction with comprehensive testing

### Distributed Generation
- **`distributed.rs`** - Field-level distributed extraction using synchronous API
- **`async_distributed.rs`** - Field-level distributed extraction using async API

### Force Generation (for Reasoning Models)
- **`sync_force.rs`** - Financial report extraction using force generation for models without JSON mode
- **`async_force.rs`** - Research paper extraction using async force generation for reasoning models

Run examples with:
```bash
# Basic synchronous example
cargo run --example sync

# Async example with comprehensive testing
cargo run --example async

# Distributed generation examples
cargo run --example distributed
cargo run --example async_distributed

# Force generation examples (for o1, deepseek, etc.)
cargo run --example sync_force
cargo run --example async_force

# To test with real API, set environment variables:
# For OpenAI:
export SECRETARY_OPENAI_API_BASE="https://api.openai.com/v1"
export SECRETARY_OPENAI_API_KEY="your-api-key"
export SECRETARY_OPENAI_MODEL="gpt-4"  # or "o1-preview", "deepseek-reasoner", etc.

# For Azure OpenAI:
export AZURE_OPENAI_ENDPOINT="https://your-resource.openai.azure.com"
export AZURE_OPENAI_API_KEY="your-azure-api-key"
export AZURE_OPENAI_DEPLOYMENT_ID="your-deployment-id"
export AZURE_OPENAI_API_VERSION="2024-02-15-preview"

cargo run --example async
```

## LLM Provider Setup

### OpenAI

For production use with OpenAI:

```bash
export SECRETARY_OPENAI_API_BASE="https://api.openai.com/v1"
export SECRETARY_OPENAI_API_KEY="your-openai-api-key"
export SECRETARY_OPENAI_MODEL="gpt-4"
```

In your code:
```rust
use secretary::llm_providers::openai::OpenAILLM;

let api_base = std::env::var("SECRETARY_OPENAI_API_BASE")
    .expect("SECRETARY_OPENAI_API_BASE environment variable not set");
let api_key = std::env::var("SECRETARY_OPENAI_API_KEY")
    .expect("SECRETARY_OPENAI_API_KEY environment variable not set");
let model = std::env::var("SECRETARY_OPENAI_MODEL")
    .expect("SECRETARY_OPENAI_MODEL environment variable not set");

let llm = OpenAILLM::new(&api_base, &api_key, &model)?;
```

### Azure OpenAI

For Azure OpenAI deployments:

```bash
export AZURE_OPENAI_ENDPOINT="https://your-resource.openai.azure.com"
export AZURE_OPENAI_API_KEY="your-azure-api-key"
export AZURE_OPENAI_DEPLOYMENT_ID="your-deployment-id"
export AZURE_OPENAI_API_VERSION="2024-02-15-preview"
```

In your code:
```rust
use secretary::llm_providers::azure::AzureOpenAILLM;

let endpoint = std::env::var("AZURE_OPENAI_ENDPOINT")
    .expect("AZURE_OPENAI_ENDPOINT environment variable not set");
let api_key = std::env::var("AZURE_OPENAI_API_KEY")
    .expect("AZURE_OPENAI_API_KEY environment variable not set");
let deployment_id = std::env::var("AZURE_OPENAI_DEPLOYMENT_ID")
    .expect("AZURE_OPENAI_DEPLOYMENT_ID environment variable not set");
let api_version = std::env::var("AZURE_OPENAI_API_VERSION")
    .expect("AZURE_OPENAI_API_VERSION environment variable not set");

let llm = AzureOpenAILLM::new(&endpoint, &api_key, &deployment_id, &api_version);
```

## API Reference

### Core Traits

| Trait | Purpose | Key Methods |
|-------|---------|-------------|
| `Task` | Main trait for data extraction tasks | `get_system_prompt()`, `get_system_prompts_for_distributed_generation()` |
| `GenerateData` | Synchronous LLM interaction | `generate_data()`, `force_generate_data()`, `fields_generate_data()` |
| `AsyncGenerateData` | Asynchronous LLM interaction | `async_generate_data()`, `async_force_generate_data()`, `async_fields_generate_data()` |
| `IsLLM` | LLM provider abstraction | `send_message()`, `async_send_message()`, `get_authorization_credentials()` |

### LLM Providers

| Provider | Description | Constructor |
|----------|-------------|-------------|
| `OpenAILLM` | OpenAI API compatible provider | `new(api_base, api_key, model)` |
| `AzureOpenAILLM` | Azure OpenAI service provider | `new(endpoint, api_key, deployment_id, api_version)` |

### Derive Macro (secretary-derive)

The `secretary-derive` crate provides procedural macros for automatic trait implementation:

- `#[derive(Task)]` - Automatically implements the `Task` trait with system prompt generation
- `#[task(instruction = "...")]` - Provides field-specific extraction instructions for the LLM

The derive macro generates:
- JSON schema definitions based on your struct fields
- System prompts that include field instructions
- Automatic `Default` trait implementation (no manual derive needed)
- Default implementations for the `Task` trait

**Note**: As of version 0.3.70, the `Default` trait is automatically implemented by the derive macro. You no longer need to include `Default` in your derive list. If you're upgrading from a previous version, simply remove `Default` from your `#[derive(...)]` declarations.

## Error Handling

`secretary` uses a comprehensive error enum, `SecretaryError`, to handle various issues that can arise during data extraction. A particularly important variant is `FieldDeserializationError`.

### `FieldDeserializationError`

This error occurs when the LLM returns data that cannot be deserialized into the target struct's field type (e.g., providing a string for a `u32` field). The error provides detailed context:

- `failed_fields`: A list of fields that failed to deserialize.
- `successful_fields`: A list of fields that were parsed correctly.
- `original_error`: The underlying error from `serde_json`.

This makes it much easier to debug issues, especially when using distributed generation.

## Troubleshooting

### Common Issues

**"Failed to execute function" Error**
- Check your API key and endpoint configuration
- Verify network connectivity
- Ensure the model name is correct

**Serialization Errors**
- Ensure all data fields implement `Serialize` and `Deserialize`
- Check that field types match the expected JSON structure
- Verify that optional fields are properly handled
- If you receive a `FieldDeserializationError`, check the following:
  - The `instruction` for the failed field. It might not be specific enough.
  - The data type of the field in your struct. It might not match what the LLM is returning.

### Performance Tips

- Use async methods for concurrent processing
- Batch multiple requests when possible
- Consider caching LLM responses for repeated queries
- Use specific field instructions to improve extraction accuracy

## Roadmap

- [x] Azure OpenAI support (✅ Completed in v0.3.60)
- [ ] Support for additional LLM providers (AWS, Anthropic, Cohere, etc.)
- [X] Enhanced error handling and validation
- [ ] Performance optimizations and caching
- [ ] Integration with more serialization formats
- [ ] Advanced prompt engineering features
- [ ] Streaming response support

### Dependencies

- **Core**: `serde`, `serde_json`, `reqwest`, `tokio`, `async-trait`
- **Derive**: `proc-macro2`, `quote`, `syn`
- **Parsing**: `surfing` (for force generation with reasoning models)

## Contributing

Contributions are welcome! 

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
