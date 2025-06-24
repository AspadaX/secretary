# Secretary Derive Macro

The Secretary derive macro simplifies the process of creating data extraction tasks by automatically implementing the `Task` trait for your data structures.

## Overview

With the new derive macro system, you no longer need to manually create `BasicTask` or `ContextualTask` instances. Instead, you can directly derive the `Task` trait on your data structures and use them immediately with LLM providers.

## Basic Usage

### 1. Define Your Data Structure

```rust
use secretary::{Task, MessageList};
use serde::{Serialize, Deserialize};

#[derive(Task, Serialize, Deserialize, Debug, Default)]
struct PersonData {
    // Required fields for Task trait (always include these)
    #[serde(skip)]
    pub context: MessageList,
    #[serde(skip)]
    pub additional_instructions: Vec<String>,
    
    // Your data fields
    #[task(instruction = "Extract the person's full name")]
    pub name: String,
    
    #[task(instruction = "Extract the age as a number")]
    pub age: u32,
    
    #[task(instruction = "Extract email if available")]
    pub email: Option<String>,
}
```

### 2. Use With LLM Providers

```rust
use secretary::llm_providers::openai::OpenAILLM;
use secretary::traits::GenerateJSON;

fn main() -> anyhow::Result<()> {
    // Create your task with additional instructions
    let task = PersonData::new(vec![
        "Be accurate with the extraction".to_string(),
        "Use null for missing information".to_string(),
    ]);
    
    // Create LLM instance
    let llm = OpenAILLM::new("api_base", "api_key", "model")?;
    
    // Generate JSON from text
    let text = "John Doe is 25 years old. Email: john@example.com";
    let result = llm.generate_json(&task, text)?;
    
    println!("Result: {}", result);
    Ok(())
}
```

## Required Fields

Every struct that derives `Task` must include these two fields:

```rust
#[serde(skip)]
pub context: MessageList,
#[serde(skip)]
pub additional_instructions: Vec<String>,
```

These fields are:
- `context`: Manages conversation history for multi-turn interactions
- `additional_instructions`: Stores custom instructions passed during creation
- Both are marked with `#[serde(skip)]` to exclude them from JSON serialization

## Field Instructions

You can provide specific extraction instructions for each field using the `#[task(instruction = "...")]` attribute:

```rust
#[task(instruction = "Extract the person's full name from the text")]
pub name: String,

#[task(instruction = "Extract age as a number, default to 0 if not found")]
pub age: u32,
```

If no instruction is provided, a default instruction will be generated based on the field name.

## Context Management

The derived `Task` trait provides context management for multi-turn conversations:

```rust
let mut task = PersonData::new(vec![]);

// Add user message
task.push(Role::User, "Tell me about John Doe")?;

// Add assistant response
task.push(Role::Assistant, "John Doe is a software engineer...")?;

// Generate with context
let result = llm.generate_json_with_context(&task)?;
```

## Migration from Old System

### Before (Old BasicTask)

```rust
use secretary::tasks::basic_task::BasicTask;
use secretary::traits::{DataModel, GenerateJSON};

#[derive(Debug, Serialize, Deserialize)]
struct MyData {
    name: String,
    age: u32,
}

impl DataModel for MyData {
    fn provide_data_model_instructions() -> Self {
        Self {
            name: "Extract the name".to_string(),
            age: 0, // placeholder
        }
    }
}

// Usage
let task = BasicTask::new::<MyData>(vec!["additional instruction".to_string()]);
let result = llm.generate_json(&task, text)?;
```

### After (New Derive Macro)

```rust
use secretary::{Task, MessageList};

#[derive(Task, Serialize, Deserialize, Debug, Default)]
struct MyData {
    #[serde(skip)]
    pub context: MessageList,
    #[serde(skip)]
    pub additional_instructions: Vec<String>,
    
    #[task(instruction = "Extract the name")]
    pub name: String,
    
    #[task(instruction = "Extract the age")]
    pub age: u32,
}

// Usage
let task = MyData::new(vec!["additional instruction".to_string()]);
let result = llm.generate_json(&task, text)?;
```

## Benefits

1. **Simplified API**: No need for separate task structs
2. **Type Safety**: Direct use of your data structures
3. **Better Documentation**: Instructions are co-located with field definitions
4. **Reduced Boilerplate**: Automatic trait implementations
5. **Consistent Interface**: All derived structs work the same way

## Advanced Features

### Custom Default Implementation

You can provide custom default values:

```rust
#[derive(Task, Serialize, Deserialize, Debug)]
struct CustomData {
    #[serde(skip)]
    pub context: MessageList,
    #[serde(skip)]
    pub additional_instructions: Vec<String>,
    
    pub name: String,
    pub score: f64,
}

impl Default for CustomData {
    fn default() -> Self {
        Self {
            context: MessageList::new(),
            additional_instructions: Vec::new(),
            name: "Unknown".to_string(),
            score: 0.0,
        }
    }
}
```

### Async Usage

```rust
use secretary::traits::AsyncGenerateJSON;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let task = PersonData::new(vec![]);
    let llm = OpenAILLM::new("api_base", "api_key", "model")?;
    
    let result = llm.async_generate_json(&task, "text to process").await?;
    println!("Result: {}", result);
    Ok(())
}
```

## Error Handling

The derive macro will generate compile-time errors if:
- The struct doesn't have the required `context` and `additional_instructions` fields
- The struct doesn't implement `Default`, `Serialize`, or `Deserialize`
- Invalid attribute syntax is used

Make sure your struct includes all required derives and fields for successful compilation.