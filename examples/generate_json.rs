//! # Secretary - Generate JSON Example
//! 
//! This example demonstrates how to use the Secretary library to convert natural language into
//! structured JSON data using OpenAI's API. Secretary provides a simple way to transform
//! unstructured text into structured data formats based on predefined schemas.
//!
//! ## Core Concept
//!
//! Secretary works by:
//! 1. Defining a structured schema (as a serializable Rust struct)
//! 2. Sending both the schema and natural language input to an LLM
//! 3. Having the LLM generate a JSON response that conforms to your schema
//!
//! ## Basic Usage Flow
//!
//! 1. Define a serializable struct that represents your desired output structure
//! 2. Initialize an LLM client (OpenAI in this example)
//! 3. Create a task with your schema and any additional instructions
//! 4. Call generate_json() with your task and input text
//! 5. Receive structured JSON that matches your schema
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
//! cargo run --example generate_json
//! ```

use secretary::{llm_providers::openai::OpenAILLM, tasks::basic_task::BasicTask, traits::{DataModel, GenerateJSON}};
use serde::{Deserialize, Serialize};

/// Define a structure that represents the expected JSON output schema.
/// 
/// The field descriptions serve as instructions to the LLM about what
/// each field should contain. This is crucial for guiding the AI to 
/// generate appropriate values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sentiment {
    /// The sentiment field will contain the sentiment classification result.
    /// This description tells the LLM how to populate this field.
    sentiment: String,
}

impl DataModel for Sentiment {
    fn provide_data_model_instructions() -> Self {
        Self {
            sentiment: String::from(
                "rate the text in terms of their sentiments. it can be: high, low or mid.",
            ),
        }
    }
}

fn main() {
    // Initialize the OpenAI LLM client with credentials from environment variables
    let llm = OpenAILLM::new(
        &std::env::var("SECRETARY_OPENAI_API_BASE").unwrap(),
        &std::env::var("SECRETARY_OPENAI_API_KEY").unwrap(),
        &std::env::var("SECRETARY_OPENAI_MODEL").unwrap(),
    )
    .unwrap();

    // Create a BasicTask with:
    // 1. The Sentiment schema that defines the expected output structure
    // 2. Additional instructions to guide the model's analysis
    //
    // These instructions help ensure more accurate and consistent results
    // by providing the LLM with specific guidance beyond just the schema.
    let task = BasicTask::new::<Sentiment>(
        vec![
            "Consider the context when determining sentiment.".to_string(),
            "Pay attention to intensifiers and negations that might change sentiment.".to_string(),
            "Some phrases may be sarcastic - look for tone indicators.".to_string(),
            "Cultural references might influence how sentiment should be interpreted.".to_string(),
            "Short texts may need more careful analysis as they contain less information."
                .to_string(),
        ],
    );

    // Generate a JSON response by providing:
    // 1. The task (containing our schema and instructions)
    // 2. The text input to analyze
    //
    // The LLM will process the input and return structured JSON that matches our schema
    let result: String = llm.generate_json(&task, "This is unacceptable!").unwrap();

    // Print the resulting JSON
    println!("Generated JSON result:");
    println!("{}", result);

    // The output will be a JSON string matching our Sentiment structure:
    // {"sentiment":"low"}
    
    // You can then parse this JSON into your struct if needed:
    // let parsed_result: Sentiment = serde_json::from_str(&result).unwrap();
}