# Getting Started with Secretary

Welcome to Secretary! This guide will walk you through setting up and using Secretary to transform natural language into structured data with LLMs.

## 📋 Prerequisites

Before you begin, you'll need:

- Rust and Cargo installed on your system
- An API key for OpenAI (or compatible provider)
- Basic familiarity with Rust and JSON

## 🚀 Installation

Add Secretary to your project's `Cargo.toml`:

```toml
[dependencies]
secretary = "0.1.12"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## 🔑 Environment Setup

Set up your environment variables for the OpenAI API:

```bash
export SECRETARY_OPENAI_API_BASE="https://api.openai.com/v1"
export SECRETARY_OPENAI_API_KEY="your-api-key"
export SECRETARY_OPENAI_MODEL="gpt-4o-mini"  # or another compatible model
```

## 🏗️ Basic Usage Pattern

Using Secretary follows a consistent pattern:

1. **Define your schema** - Create a Rust struct for your data
2. **Initialize an LLM client** - Set up connection to the AI provider
3. **Create a task** - Combine schema with instructions
4. **Extract structured data** - Process natural language input

## 🧩 Step-by-Step Example

### 1. Define Your Schema

Start by defining the structure you want to extract:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Contact {
    name: String,
    email: String,
    phone_number: Option<String>,
    preferred_contact_method: String,
}

impl Default for Contact {
    fn default() -> Self {
        Self {
            name: "The person's full name".to_string(),
            email: "The person's email address".to_string(),
            phone_number: Some("The person's phone number, if mentioned".to_string()),
            preferred_contact_method: "How they prefer to be contacted: email, phone, or other".to_string(),
        }
    }
}
```

### 2. Initialize the LLM Client

Connect to your chosen LLM provider:

```rust
use secretary::openai::OpenAILLM;

fn main() -> anyhow::Result<()> {
    let llm = OpenAILLM::new(
        &std::env::var("SECRETARY_OPENAI_API_BASE").unwrap_or("https://api.openai.com/v1".to_string()),
        &std::env::var("SECRETARY_OPENAI_API_KEY").unwrap(),
        &std::env::var("SECRETARY_OPENAI_MODEL").unwrap_or("gpt-4o-mini".to_string()),
    )?;
    
    // Continue with the next steps...
    Ok(())
}
```

### 3. Create a Task

Set up a task with your schema and processing instructions:

```rust
use secretary::tasks::basic_task::BasicTask;

let task = BasicTask::new(
    Contact::default(),
    vec![
        "Extract contact information from the text".to_string(),
        "If a contact method isn't explicitly preferred, choose based on what's provided".to_string(),
        "Only include phone_number if it's explicitly mentioned".to_string(),
    ],
);
```

### 4. Process Natural Language Input

Use the `generate_json` method to extract data from text:

```rust
use secretary::traits::GenerateJSON;

let input = "Hi, my name is John Doe. You can reach me at john.doe@example.com. \
             I prefer to be contacted by email rather than calling.";

let json_result = llm.generate_json(&task, input)?;
println!("Extracted contact info: {}", json_result);
```

The result will be structured JSON:

```json
{
  "name": "John Doe",
  "email": "john.doe@example.com",
  "phone_number": null,
  "preferred_contact_method": "email"
}
```

## 🔄 Working with the Results

Parse the JSON into your defined struct:

```rust
let contact: Contact = serde_json::from_str(&json_result)?;
println!("Contact name: {}", contact.name);
println!("Contact email: {}", contact.email);
```

## 🌟 Next Steps

- Check out the [examples directory](../examples) for more usage patterns
- Learn about [multi-turn conversations](EXAMPLES.md#multi-turn-conversations)
- Explore [contextual tasks](EXAMPLES.md#contextual-tasks) for complex extractions

## 🔧 Troubleshooting

- **JSON Parsing Errors**: Ensure your schema matches the expected structure
- **API Errors**: Verify your API credentials and environment variables
- **Missing Information**: Try enhancing your task instructions to guide the LLM

For more details, refer to the [official documentation](https://docs.rs/secretary).