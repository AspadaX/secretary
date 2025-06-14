//! # Secretary - Interactive Contextual Prompt Conversation Example
//! 
//! This example demonstrates Secretary's ContextualTask for multi-turn conversations.
//! It showcases how to use the AI's reasoning, notes, and follow-up questions to build
//! a progressive, context-aware conversation with structured data extraction.
//!
//! ## Conversation Flow with ContextualTask
//!
//! This example shows:
//! 1. **Progressive reasoning**: The AI builds understanding across turns
//! 2. **Note accumulation**: Important points are collected throughout the conversation
//! 3. **Contextual follow-ups**: The AI asks relevant questions based on previous answers
//! 4. **Structured data extraction**: Information is organized into the schema
//!
//! ## Multi-Turn Process
//!
//! 1. Define your data schema with all fields you eventually want to extract
//! 2. Create a ContextualTask with instructions for progressive extraction
//! 3. Process each user message, examining the AI's reasoning and follow-ups
//! 4. Continue the conversation, with the AI refining its understanding
//! 5. End with a complete structured data object built across turns
//!
//! ## Environment Variables
//!
//! Before running this example, set:
//! - `SECRETARY_OPENAI_API_BASE`: Your OpenAI API base URL
//! - `SECRETARY_OPENAI_API_KEY`: Your OpenAI API key
//! - `SECRETARY_OPENAI_MODEL`: The model to use (e.g., "gpt-4o-mini")
//!
//! ```bash
//! export SECRETARY_OPENAI_API_BASE="https://api.openai.com/v1"
//! export SECRETARY_OPENAI_API_KEY="your-api-key"
//! export SECRETARY_OPENAI_MODEL="gpt-4o-mini"
//! cargo run --example contextual_prompt_conversation
//! ```

use std::io::{self, Write};
use anyhow::Result;
use secretary::{
    message_list::Role,
    openai::OpenAILLM,
    tasks::contextual_task::ContextualTask,
    traits::{Context, DataModel, GenerateJSON},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Trip planning schema with fields populated through conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TripPlan {
    /// Destination city or location
    destination: String,
    
    /// Date range for the trip (e.g., "June 10-15, 2023")
    date_range: String,
    
    /// Budget range in USD
    budget: Option<String>,
    
    /// Number of travelers
    travelers: Option<u8>,
    
    /// Accommodation preferences
    accommodation_preferences: Option<Vec<String>>,
    
    /// Must-see attractions
    attractions: Vec<String>,
    
    /// Dietary restrictions or preferences
    dietary_preferences: Option<Vec<String>>,
    
    /// Special activities or experiences
    special_activities: Option<Vec<String>>,
    
    /// Transportation preferences during the trip
    local_transportation: Option<String>,
}

impl DataModel for TripPlan {
    fn provide_data_model_instructions() -> Self {
         Self {
            destination: String::from("Destination city or location that the user would like to go to"),
            date_range: String::from("Date range for the user's trip"),
            budget: Some(String::from("The user's budget range in USD")),
            travelers: None,
            accommodation_preferences: Some(vec![String::from("The user's preferred accommodation types")]),
            attractions: vec![String::from("Must-see attractions the user wants to visit")],
            dietary_preferences: Some(vec![String::from("Any dietary restrictions or preferences of the user")]),
            special_activities: Some(vec![String::from("Special activities or experiences the user wants")]),
            local_transportation: Some(String::from("The user's transportation preferences during the trip")),
        }
    }
}

/// Format and display the current trip plan details from the response
fn display_trip_plan(response_json: &Value) {
    println!("\n=== Current Trip Plan ===");
    
    if let Some(data) = response_json.get("data_structure") {
        // Helper function to print a field if available
        let print_field = |name: &str, field: &str| {
            if let Some(value) = data.get(field) {
                if !value.is_null() {
                    match value {
                        Value::String(s) if !s.is_empty() => println!("{}: {}", name, s),
                        Value::Number(n) => println!("{}: {}", name, n),
                        Value::Array(arr) if !arr.is_empty() => {
                            println!("{}:", name);
                            for item in arr {
                                if let Some(item_str) = item.as_str() {
                                    println!("  - {}", item_str);
                                }
                            }
                        },
                        _ => {}
                    }
                }
            }
        };
        
        print_field("Destination", "destination");
        print_field("Date Range", "date_range");
        print_field("Budget", "budget");
        print_field("Travelers", "travelers");
        print_field("Accommodation", "accommodation_preferences");
        print_field("Attractions", "attractions");
        print_field("Dietary Preferences", "dietary_preferences");
        print_field("Special Activities", "special_activities");
        print_field("Local Transportation", "local_transportation");
    }
    
    println!("========================\n");
}

/// Read user input from the console
fn read_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    input.trim().to_string()
}

fn main() -> Result<()> {
    println!("🌟 Trip Planning Assistant with ContextualTask 🌟");
    println!("This example shows a progressive conversation that builds a structured trip plan.");
    println!("Type 'exit' at any time to end the conversation.\n");
    
    // Initialize the OpenAI LLM client
    let llm = OpenAILLM::new(
        &std::env::var("SECRETARY_OPENAI_API_BASE").unwrap_or("https://api.openai.com/v1".to_string()),
        &std::env::var("SECRETARY_OPENAI_API_KEY").unwrap(),
        &std::env::var("SECRETARY_OPENAI_MODEL").unwrap_or("gpt-4o-mini".to_string()),
    )?;
    
    // Create a contextual prompt with our trip planning schema
    let mut prompt = ContextualTask::new::<TripPlan>(
        vec![
            "Help the user plan a trip by gathering information progressively.".to_string(),
            "Update the data_structure as you learn details about their trip plans.".to_string(),
            "Ask one focused question at a time to gather missing information.".to_string(),
            "When information is incomplete, leave those fields as null.".to_string(),
            "Number of travellers is null in the example, but you should ask the user for this.".to_string(),
        ],
    );
    
    // Start the conversation with an initial user prompt
    prompt.push(Role::User, "I'm planning a trip to Japan next spring and need help organizing the details.")?;
    
    // Main conversation loop
    let mut turn: i32 = 1;
    loop {
        println!("\n--- Turn {} ---", turn);
        
        // Generate JSON response from the contextual prompt
        let json_response = llm.generate_json_with_context(&prompt)?;
        
        // Parse the response
        let response_value: Value = serde_json::from_str(&json_response)?;
        
        // Display AI reasoning
        if let Some(reasoning) = response_value.get("reasoning").and_then(|v| v.as_str()) {
            println!("\n🤔 AI's Reasoning Process:\n{}", reasoning);
        }
        
        // Display accumulated notes
        if let Some(notes) = response_value.get("notes").and_then(|v| v.as_array()) {
            if !notes.is_empty() {
                println!("\n📝 Key Notes:");
                for (i, note) in notes.iter().enumerate() {
                    if let Some(note_str) = note.as_str() {
                        println!("  {}. {}", i+1, note_str);
                    }
                }
            }
        }
        
        // Display current trip plan status
        display_trip_plan(&response_value);
        
        // Display AI's response and follow-up question
        if let Some(content) = response_value.get("content").and_then(|v| v.as_str()) {
            println!("Assistant: {}", content);
        } else {
            println!("Assistant: I think we've completed your trip plan! Is there anything else you'd like to add or modify?");
        }
        
        // Add AI's response to conversation context
        prompt.push(Role::Assistant, &json_response)?;
        
        // Get user's next input
        let user_input: String = read_user_input("\nYour response: ");
        
        // Check if user wants to exit
        if user_input.to_lowercase() == "exit" {
            println!("\nFinal trip plan:");
            display_trip_plan(&response_value);
            println!("Thanks for using the Trip Planning Assistant!");
            break;
        }
        
        // Add user's response to conversation context
        prompt.push(Role::User, &user_input)?;
        
        // Increment turn counter
        turn += 1;
    }
    
    Ok(())
}