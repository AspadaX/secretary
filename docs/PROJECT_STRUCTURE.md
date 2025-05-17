# Secretary Project Structure

This document provides an overview of the Secretary project's architecture and organization to help developers understand how the code is structured.

## Overview

Secretary is designed with a modular architecture that separates concerns and enables extensibility. The main components are:

- **Traits**: Core interfaces that define functionality
- **LLM Providers**: Implementations for specific AI providers
- **Tasks**: Schema definitions with instructions for the LLM
- **Message Management**: Handling conversation context

## Directory Structure

```
secretary/
├── src/
│   ├── lib.rs            # Main library entry point
│   ├── traits.rs         # Core interfaces and shared functionality
│   ├── message_list.rs   # Message and conversation management
│   ├── openai.rs         # OpenAI API implementation
│   └── tasks/            # Task implementations
│       ├── mod.rs        # Task module exports
│       ├── basic_task.rs # Simple task implementation
│       └── contextual_task.rs # Advanced contextual task implementation
├── examples/             # Example applications
└── docs/                # Documentation
```

## Core Components

### Traits (`traits.rs`)

This file defines the foundational interfaces that power Secretary:

- **`IsLLM`**: Interface for LLM provider implementations
- **`SystemPrompt`**: Interface for objects that have system prompts
- **`Context`**: Interface for managing conversation context
- **`GenerateJSON`**: Functionality for generating JSON from an LLM
- **`AsyncGenerateJSON`**: Async version of the JSON generation functionality

### Message Management (`message_list.rs`)

Handles the message data structure and conversation flow:

- **`Role`**: Enum for message roles (System, User, Assistant)
- **`Message`**: Structure for individual messages
- **`MessageList`**: Collection for managing conversation history

### LLM Provider (`openai.rs`)

Implementation for specific LLM providers:

- **`OpenAILLM`**: Implementation of the `IsLLM` trait for OpenAI API
- Client configuration and API handling

### Tasks

Two primary task implementations:

#### Basic Task (`tasks/basic_task.rs`)

- Simple task with a data structure and instructions
- Suitable for single-turn extractions without complex context

#### Contextual Task (`tasks/contextual_task.rs`)

- Advanced task with reasoning, notes, and dynamic content
- Maintains state across multiple conversation turns
- Accumulates observations and insights over time
- Compressed context. Less token consumption

## Data Flow

The typical data flow through Secretary is:

1. **Schema Definition**: Define your data structure with Rust structs
2. **Task Creation**: Create a task with your schema and instructions
3. **LLM Integration**: Initialize an LLM client
4. **Input Processing**: Send user input to the LLM through the task
5. **Structured Output**: Receive structured JSON that matches your schema

## Extension Points

Secretary is designed to be extensible:

- **New LLM Providers**: Implement the `IsLLM` trait for other AI providers
- **Custom Tasks**: Create specialized task types by implementing `SystemPrompt` and `Context`
- **Processing Pipelines**: Build on the JSON output with custom post-processing

## Design Principles

Secretary follows these design principles:

1. **Type Safety**: Leveraging Rust's type system for robust data handling
2. **Declarative Schemas**: Using structs and field annotations to define extraction targets
3. **Separation of Concerns**: Clear boundaries between LLM access, schema definition, and context management
4. **Progressive Enhancement**: Simple interfaces for basic needs, with more advanced options available

## Testing

For testing Secretary implementations:

- Unit tests for individual components
- Integration tests that verify end-to-end functionality
- Mock LLM providers for testing without API calls

## Future Directions

Planned extensions to the library:

- Support for additional LLM providers
- Streaming response support
- Schema validation and refinement