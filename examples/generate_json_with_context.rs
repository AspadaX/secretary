//! # Secretary - Generate JSON with Multi-Turn Conversation Example
//!
//! This example demonstrates how to use Secretary for generating structured JSON through a
//! multi-turn conversation. Unlike the basic example, this maintains conversation state
//! between turns, enabling context-aware responses.
//!
//! ## How Contextual Processing Works
//!
//! Secretary's contextual processing:
//! 1. Maintains a conversation history using the `Context` trait
//! 2. Sends the entire conversation context to the LLM with each request
//! 3. Ensures each response considers previous interactions
//! 4. Automatically formats responses as JSON matching your schema
//!
//! ## Multi-Turn Conversation Flow
//!
//! 1. Define your output schema as a serializable struct
//! 2. Create a BasicTask with your schema and instructions
//! 3. Add user messages to the context
//! 4. Generate a JSON response
//! 5. Add the response back to the context as an assistant message
//! 6. Continue the conversation by adding more user messages
//!
//! ## Environment Variables
//!
//! Before running this example, set the following environment variables:
//! - `SECRETARY_OPENAI_API_BASE`: Your OpenAI API base URL (usually "https://api.openai.com/v1")
//! - `SECRETARY_OPENAI_API_KEY`: Your OpenAI API key
//! - `SECRETARY_OPENAI_MODEL`: The model to use (e.g., "gpt-4o-mini" or "gpt-4o")
//!
//! ```bash
//! export SECRETARY_OPENAI_API_BASE="https://api.openai.com/v1"
//! export SECRETARY_OPENAI_API_KEY="your-api-key"
//! export SECRETARY_OPENAI_MODEL="gpt-4o-mini"
//! cargo run --example generate_json_with_context
//! ```

use secretary::{
    message_list::Role, 
    llm_providers::openai::OpenAILLM, 
    tasks::basic_task::BasicTask, 
    traits::{Context, DataModel, GenerateJSON},
};
use serde::{Deserialize, Serialize};

/// Schema defining the expected JSON output structure.
/// 
/// In a real application, this would be customized to your specific data needs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sentiment {
    /// The sentiment classification field with guidance for the LLM.
    sentiment: String,
    
    /// Optional explanation of the sentiment rating.
    /// This provides context for why a particular rating was given.
    explanation: Option<String>,
}

impl DataModel for Sentiment {
    fn provide_data_model_instructions() -> Self {
        Self {
            sentiment: String::from(
                "rate the text in terms of their sentiments. it can be: high, low or mid.",
            ),
            explanation: Some(String::from(
                "provide a brief explanation of why you gave this sentiment rating"
            ))
        }
    }
}

fn main() {
    // Initialize the OpenAI LLM client with credentials from environment variables
    let llm = OpenAILLM::new(
        &std::env::var("SECRETARY_OPENAI_API_BASE").unwrap_or("https://api.openai.com/v1".to_string()),
        &std::env::var("SECRETARY_OPENAI_API_KEY").unwrap(),
        &std::env::var("SECRETARY_OPENAI_MODEL").unwrap_or("gpt-4o-mini".to_string()),
    )
    .unwrap();

    // Create a BasicTask with our schema and analysis instructions
    let mut task = BasicTask::new::<Sentiment>(
        vec![
            "Consider the full conversation context when determining sentiment.".to_string(),
            "Pay attention to intensifiers and negations that might change sentiment.".to_string(),
            "Look for changes in sentiment across multiple messages.".to_string(),
            "The explanation should be brief but informative.".to_string(),
        ],
    );

    // Define a series of user messages to simulate a conversation
    let conversation: Vec<&'static str> = vec![
        "I just got my new phone today.",
        "The battery life is much better than expected!",
        "But the camera quality is disappointing.",
        "After using it for a few hours, I think it's worth the money overall.",
    ];

    println!("Starting a multi-turn conversation...\n");

    // Simulate a multi-turn conversation
    for (i, message) in conversation.iter().enumerate() {
        println!("Turn {}: User says: \"{}\"", i+1, message);
        
        // Add the user message to the conversation context
        task.push(Role::User, message).unwrap();
        
        // Generate a JSON response based on the entire conversation context
        let result: String = llm.generate_json_with_context(&task).unwrap();
        
        // Parse and pretty-print the JSON result
        let value: serde_json::Value = serde_json::from_str(&result).unwrap();
        let pretty_json = serde_json::to_string_pretty(&value).unwrap();
        
        println!("Turn {}: Assistant response:", i+1);
        println!("{}\n", pretty_json);
        
        // Add the AI's response back to the conversation context
        // This ensures future responses will be aware of this exchange
        task.push(Role::Assistant, &result).unwrap();
    }
    
    println!("Conversation complete!");
    
    // The final sentiment analysis will consider the entire conversation,
    // showing how the sentiment evolved over multiple turns.
}