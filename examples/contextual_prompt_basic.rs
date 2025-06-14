//! # Secretary - Basic Contextual Prompt Example
//! 
//! This example demonstrates how to use Secretary's ContextualTask for creating
//! structured AI interactions with reasoning and explanations. Unlike BasicTask,
//! ContextualTask includes fields for the AI's reasoning process, optional follow-up
//! questions, and progressive note-taking.
//!
//! ## ContextualTask Structure
//!
//! The ContextualTask contains:
//! 1. **reasoning**: The AI's thought process and analysis
//! 2. **content**: Optional follow-up questions or requests for more information
//! 3. **notes**: Key points to track throughout the conversation
//! 4. **data_structure**: Your custom data schema (using serde_json::Value)
//!
//! ## Basic Usage Flow
//!
//! 1. Define your data schema
//! 2. Create a ContextualTask with your schema and instructions
//! 3. Process user input to generate structured responses with reasoning
//! 4. Access the AI's reasoning, content, notes, and extracted data
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
//! cargo run --example contextual_prompt_basic
//! ```

use anyhow::Result;
use secretary::{
    message_list::Role,
    llm_providers::openai::OpenAILLM,
    tasks::contextual_task::ContextualTask,
    traits::{Context, DataModel, GenerateJSON},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Product analysis schema with custom fields for extracted information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductAnalysis {
    /// Product category (e.g., electronics, kitchen, furniture)
    category: String,
    
    /// Target audience for this product
    target_audience: Vec<String>,
    
    /// Key features worth highlighting
    key_features: Vec<String>,
    
    /// Estimated price range (low, medium, high)
    price_range: String,
    
    /// Competitive advantage over similar products
    competitive_advantage: Option<String>,
}

impl DataModel for ProductAnalysis {
    fn provide_data_model_instructions() -> Self {
        Self {
            category: String::from("product category"),
            target_audience: vec![String::from("example audience group")],
            key_features: vec![String::from("example feature")],
            price_range: String::from("low, medium, or high"),
            competitive_advantage: None,
        }
    }
}

fn main() -> Result<()> {
    // Initialize the OpenAI LLM client
    let llm = OpenAILLM::new(
        &std::env::var("SECRETARY_OPENAI_API_BASE").unwrap_or("https://api.openai.com/v1".to_string()),
        &std::env::var("SECRETARY_OPENAI_API_KEY").unwrap(),
        &std::env::var("SECRETARY_OPENAI_MODEL").unwrap_or("gpt-4o-mini".to_string()),
    )?;
    
    // Create a contextual prompt with our product analysis schema
    let mut prompt: ContextualTask = ContextualTask::new::<ProductAnalysis>(
        vec![
            "Analyze the product description in detail.".to_string(),
            "Identify the likely category and target audience based on features.".to_string(),
            "Extract key product features and selling points.".to_string(),
            "Estimate price range based on similar products.".to_string(),
            "In your reasoning, explain how you arrived at each conclusion.".to_string(),
            "If information is insufficient, note this in your reasoning.".to_string(),
        ],
    );
    
    // Add user message with a product description
    prompt.push(Role::User, "Our new smart water bottle tracks your daily hydration, syncs with fitness apps, and gently reminds you to drink water with subtle light patterns. It's made from BPA-free materials, keeps drinks cold for 24 hours, and has a battery life of 2 weeks. The companion app provides personalized hydration goals based on your activity level and local weather.")?;
    
    // Generate a structured response with reasoning and analysis
    let json_response: String = llm.generate_json_with_context(&prompt)?;
    
    // Parse the JSON response
    let value: Value = serde_json::from_str(&json_response)?;
    let pretty_json: String = serde_json::to_string_pretty(&value)?;
    
    // Display the structured response
    println!("# ContextualTask Response\n");
    println!("{}\n", pretty_json);
    
    // Extract key components from the response
    if let Some(reasoning) = value.get("reasoning").and_then(|v| v.as_str()) {
        println!("## AI Reasoning Process:\n{}\n", reasoning);
    }
    
    if let Some(notes) = value.get("notes").and_then(|v| v.as_array()) {
        println!("## Key Notes:");
        for (i, note) in notes.iter().enumerate() {
            if let Some(note_str) = note.as_str() {
                println!("{}. {}", i+1, note_str);
            }
        }
        println!();
    }
    
    if let Some(content) = value.get("content").and_then(|v| v.as_str()) {
        println!("## Follow-up Questions:\n{}\n", content);
    }
    
    if let Some(data) = value.get("data_structure") {
        println!("## Extracted Product Analysis Data:");
        
        if let Some(category) = data.get("category").and_then(|v| v.as_str()) {
            println!("Category: {}", category);
        }
        
        if let Some(audience) = data.get("target_audience").and_then(|v| v.as_array()) {
            print!("Target Audience: ");
            for (i, group) in audience.iter().enumerate() {
                if let Some(group_str) = group.as_str() {
                    print!("{}{}", group_str, if i < audience.len() - 1 { ", " } else { "" });
                }
            }
            println!();
        }
        
        if let Some(features) = data.get("key_features").and_then(|v| v.as_array()) {
            println!("Key Features:");
            for feature in features {
                if let Some(feature_str) = feature.as_str() {
                    println!("- {}", feature_str);
                }
            }
        }
        
        if let Some(price) = data.get("price_range").and_then(|v| v.as_str()) {
            println!("Price Range: {}", price);
        }
        
        if let Some(advantage) = data.get("competitive_advantage").and_then(|v| v.as_str()) {
            println!("Competitive Advantage: {}", advantage);
        }
    }
    
    Ok(())
}