//! # Secretary - Async JSON Generation with Context Example
//! 
//! This example demonstrates Secretary's async API for concurrent multi-turn conversations.
//! It simulates multiple parallel conversations, each maintaining its own context, processed
//! concurrently using Tokio's async runtime.
//!
//! ## Async Context Processing Benefits
//!
//! Using async processing with contextual conversations:
//! 1. **Handle multiple conversations**: Process multiple independent conversations simultaneously
//! 2. **Efficient resource usage**: Non-blocking I/O improves throughput and resource utilization
//! 3. **Scalability**: Easily scale to handle many concurrent conversations
//! 4. **Reduced overall latency**: Process multiple conversation turns in parallel
//!
//! ## Async Contextual Processing Flow
//!
//! 1. Define a serializable schema for structured output
//! 2. Create separate tasks for each conversation context 
//! 3. Spawn async tasks for independent conversation processing
//! 4. Update conversation contexts asynchronously as responses arrive
//! 5. Display results showing separate conversation threads
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
//! cargo run --example async_generate_json_with_context
//! ```

use std::sync::Arc;
use std::time::Instant;
use anyhow::Result;
use secretary::{
    message_list::Role,
    llm_providers::openai::OpenAILLM,
    tasks::basic_task::BasicTask,
    traits::{AsyncGenerateJSON, Context, DataModel},
};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use std::time::Duration;

/// Schema defining the expected JSON output structure for product inquiries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductInquiry {
    /// Answer to the customer's question
    answer: String,
    
    /// Confidence level in the answer (high, medium, low)
    confidence: String,
    
    /// Whether more information is needed from the customer
    needs_more_info: bool,
    
    /// Follow-up questions to ask the customer if applicable
    follow_up_questions: Option<Vec<String>>,
}

impl DataModel for ProductInquiry {
    fn provide_data_model_instructions() -> Self {
        Self {
            answer: String::new(),
            confidence: String::from("high, medium, or low"),
            needs_more_info: false,
            follow_up_questions: None,
        }
    }
}

/// Simulated customer conversations with multiple turns
fn get_customer_conversations() -> Vec<Vec<&'static str>> {
    vec![
        // Customer 1 - Inquiring about smartphone features
        vec![
            "Does the X2000 smartphone have wireless charging?",
            "How does it compare to the previous model?",
            "Is it worth upgrading if I have last year's model?",
        ],
        
        // Customer 2 - Technical support issue
        vec![
            "My laptop won't turn on after the latest update.",
            "I've tried holding the power button for 30 seconds as suggested.",
            "The power light blinks three times then goes off.",
        ],
        
        // Customer 3 - Product recommendation
        vec![
            "I need headphones for running that won't fall out.",
            "I prefer something with good bass and water resistance.",
            "My budget is around $100.",
        ],
    ]
}

/// Process a complete multi-turn conversation asynchronously
async fn process_conversation(
    llm: Arc<OpenAILLM>,
    conversation_id: usize,
    messages: &[&str],
) -> Result<()> {
    println!("🟢 Starting conversation #{} processing", conversation_id);
    let start_time = Instant::now();
    
    // Create a task with our schema and instructions
    let mut task = BasicTask::new::<ProductInquiry>(
        vec![
            "Provide helpful, accurate responses to customer product inquiries.".to_string(),
            "If you're uncertain, indicate lower confidence and ask for more details.".to_string(),
            "For product comparisons, highlight key differences in features.".to_string(),
            "For technical support, provide clear troubleshooting steps.".to_string(),
        ],
    );
    
    // Process each message in the conversation sequentially
    // (each response depends on previous context)
    for (turn, message) in messages.iter().enumerate() {
        // Add a slight delay to simulate realistic conversation timing
        if turn > 0 {
            sleep(Duration::from_millis(500)).await;
        }
        
        println!("\n🔹 Conversation #{} - Turn {}", conversation_id, turn + 1);
        println!("Customer: \"{}\"", message);
        
        // Add the user message to the conversation context
        task.push(Role::User, message)?;
        
        // Generate a JSON response based on the entire conversation context
        let result = llm.async_generate_json_with_context(&task).await?;
        
        // Parse and pretty-print the JSON result
        let value: serde_json::Value = serde_json::from_str(&result)?;
        let pretty_json = serde_json::to_string_pretty(&value)?;
        
        println!("Assistant response:");
        println!("{}", pretty_json);
        
        // Add the response back to the conversation context
        task.push(Role::Assistant, &result)?;
    }
    
    println!("\n✅ Conversation #{} completed in {:.2?}", 
             conversation_id, start_time.elapsed());
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let overall_start = Instant::now();
    
    // Initialize the OpenAI LLM client with credentials from environment variables
    // Wrap in Arc for thread-safe sharing across async tasks
    let llm = Arc::new(OpenAILLM::new(
        &std::env::var("SECRETARY_OPENAI_API_BASE").unwrap_or("https://api.openai.com/v1".to_string()),
        &std::env::var("SECRETARY_OPENAI_API_KEY").unwrap(),
        &std::env::var("SECRETARY_OPENAI_MODEL").unwrap_or("gpt-4o-mini".to_string()),
    )?);
    
    // Get our simulated customer conversations
    let conversations = get_customer_conversations();
    println!("Processing {} conversations concurrently...\n", conversations.len());
    
    // Create a vector to hold our tasks
    let mut handles = Vec::with_capacity(conversations.len());
    
    // Launch concurrent tasks for each conversation
    for (i, messages) in conversations.iter().enumerate() {
        // Clone the Arc to the LLM for this task
        let llm_clone = Arc::clone(&llm);
        let conversation_id = i + 1;
        let messages_clone = messages.to_vec();
        
        // Process each conversation in a separate task
        let handle = tokio::spawn(async move {
            match process_conversation(llm_clone, conversation_id, &messages_clone).await {
                Ok(_) => println!("Conversation #{} processed successfully", conversation_id),
                Err(e) => eprintln!("Error in conversation #{}: {}", conversation_id, e),
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all conversations to complete
    for handle in handles {
        handle.await?;
    }
    
    // Print performance summary
    let total_duration = overall_start.elapsed();
    println!("\n⭐ All conversations processed in {:.2?}", total_duration);
    println!("Using async processing allowed all conversations to progress simultaneously!");
    println!("In a sequential implementation, the total time would be the sum of all conversation times.");
    
    Ok(())
}
