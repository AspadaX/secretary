//! # Secretary - Async JSON Generation Example
//! 
//! This example demonstrates how to use Secretary's async API to perform concurrent
//! JSON generation requests. By using async/await, we can efficiently process multiple
//! inputs simultaneously, significantly improving throughput compared to sequential processing.
//!
//! ## Async Processing Benefits
//!
//! Async processing provides several advantages:
//! 1. **Improved throughput**: Process multiple requests concurrently
//! 2. **Resource efficiency**: Better utilization of system resources
//! 3. **Reduced latency**: Overall processing time is closer to the slowest single request
//!    rather than the sum of all request times
//!
//! ## Basic Async Usage Flow
//!
//! 1. Define a serializable struct representing your desired output structure
//! 2. Initialize an async LLM client
//! 3. Create tasks with your schema and instructions
//! 4. Use `async_generate_json()` with `tokio::spawn()` to process requests concurrently
//! 5. Await and collect the results
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
//! cargo run --example async_generate_json
//! ```

use std::sync::Arc;
use std::time::Instant;
use anyhow::Result;
use secretary::{llm_providers::openai::OpenAILLM, tasks::basic_task::BasicTask, traits::{AsyncGenerateJSON, DataModel}};
use serde::{Deserialize, Serialize};

/// Schema defining the expected JSON output structure.
/// 
/// This example uses a product review analysis schema to demonstrate
/// structured data extraction from natural language inputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductReview {
    /// Product rating on a scale of 1-5
    rating: u8,
    
    /// Key positive aspects mentioned in the review
    pros: Vec<String>,
    
    /// Key negative aspects mentioned in the review
    cons: Vec<String>,
    
    /// Overall sentiment (positive, negative, or mixed)
    sentiment: String,
}

impl DataModel for ProductReview {
    fn provide_data_model_instructions() -> Self {
        Self {
            rating: 0,
            pros: Vec::new(),
            cons: Vec::new(),
            sentiment: String::from("positive, negative, or mixed"),
        }
    }
}

/// Sample product reviews for processing
fn get_sample_reviews() -> Vec<&'static str> {
    vec![
        "I love this phone! The camera is amazing and battery life is stellar. The only downside is the price.",
        "This laptop is terrible. It's slow, overheats constantly, and the keyboard stopped working after just two months.",
        "Mixed feelings about this headset. Great sound quality and comfort, but the microphone picks up too much background noise.",
        "The smartwatch exceeded my expectations. Accurate fitness tracking, stylish design, and excellent battery life.",
        "Disappointing purchase. The blender makes a horrible noise and doesn't blend frozen items well. Save your money.",
    ]
}

#[tokio::main]
async fn main() -> Result<()> {
    // Start timing for performance comparison
    let start = Instant::now();
    
    // Initialize the OpenAI LLM client with credentials from environment variables
    // Wrap in Arc for thread-safe sharing across async tasks
    let llm: Arc<OpenAILLM> = Arc::new(OpenAILLM::new(
        &std::env::var("SECRETARY_OPENAI_API_BASE").unwrap_or("https://api.openai.com/v1".to_string()),
        &std::env::var("SECRETARY_OPENAI_API_KEY").unwrap(),
        &std::env::var("SECRETARY_OPENAI_MODEL").unwrap_or("gpt-4o-mini".to_string()),
    )?);
    
    // Create a task template with our schema and analysis instructions
    // Also wrap in Arc for thread-safe sharing
    let task: Arc<BasicTask> = Arc::new(BasicTask::new::<ProductReview>(
        vec![
            "Extract key information from product reviews.".to_string(),
            "Rating should be an integer from 1-5 based on the reviewer's sentiment.".to_string(),
            "Pros and cons should be specific features or aspects mentioned.".to_string(),
            "Sentiment should be 'positive', 'negative', or 'mixed'.".to_string(),
        ],
    ));
    
    // Get sample product reviews to process
    let reviews = get_sample_reviews();
    println!("Processing {} reviews concurrently...\n", reviews.len());
    
    // Create a vector to hold our tasks
    let mut handles = Vec::with_capacity(reviews.len());
    
    // Launch concurrent tasks for each review
    for (i, review) in reviews.iter().enumerate() {
        // Clone Arc references to the LLM and task
        let llm_clone = Arc::clone(&llm);
        let task_clone = Arc::clone(&task);
        let review_clone = review.to_string();
        let review_index = i + 1;
        
        // Spawn a task for each review
        let handle = tokio::spawn(async move {
            println!("Starting analysis of review #{}", review_index);
            let start_time = Instant::now();
            
            // Process this specific review
            let result = llm_clone.async_generate_json(&*task_clone, &review_clone).await;
            
            (review_index, review_clone, result, start_time.elapsed())
        });
        
        handles.push(handle);
    }
    
    // Wait for all tasks to complete and process results
    for handle in handles {
        let (index, review, result, duration) = handle.await?;
        
        match result {
            Ok(json) => {
                // Parse the JSON for pretty printing
                let value: serde_json::Value = serde_json::from_str(&json)?;
                let pretty_json = serde_json::to_string_pretty(&value)?;
                
                println!("Review #{} processed in {:.2?}:", index, duration);
                println!("\"{}\"", review);
                println!("{}\n", pretty_json);
            },
            Err(e) => println!("Error processing review #{}: {}", index, e),
        }
    }
    
    // Print performance summary
    let total_duration = start.elapsed();
    println!("All reviews processed in {:.2?}", total_duration);
    println!("Using async processing provided significant performance benefits compared to sequential processing!");
    
    Ok(())
}
