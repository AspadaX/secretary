use secretary::Task;
use secretary::llm_providers::openai::OpenAILLM;
use secretary::traits::AsyncGenerateData;
use serde::{Deserialize, Serialize};
use tokio;

/// Example data structure for extracting research paper information
/// This example demonstrates force generation for LLMs without JSON mode support
#[derive(Task, Serialize, Deserialize, Debug)]
struct ResearchPaperExtraction {
    /// Research paper data fields with specific extraction instructions
    #[task(instruction = "Extract the title of the research paper")]
    pub title: String,

    #[task(instruction = "Extract the main author or first author's name")]
    pub primary_author: String,

    #[task(instruction = "Extract all co-authors as a comma-separated list")]
    pub co_authors: Option<String>,

    #[task(instruction = "Extract the publication year as a number")]
    pub year: u32,

    #[task(instruction = "Extract the journal or conference name")]
    pub venue: String,

    #[task(instruction = "Extract the abstract or summary of the paper")]
    pub abstract_text: String,

    #[task(instruction = "Extract key research topics or keywords")]
    pub keywords: Vec<String>,

    #[task(instruction = "Determine if this is peer-reviewed (true/false)")]
    pub peer_reviewed: bool,
}

/// Async force example demonstrating JSON parsing for reasoning models
/// This example shows how to use async_force_generate_data for models like o1 and deepseek
/// that don't have built-in JSON mode support but can still generate structured data
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    println!("Secretary Async Force Example - Research Paper Extraction");
    println!("{}", "=".repeat(65));
    println!("This example demonstrates force JSON parsing for reasoning models");
    println!("like o1 and deepseek that don't support native JSON mode.\n");

    // Create a task instance
    let task = ResearchPaperExtraction::new();

    // Additional instructions for the LLM
    let additional_instructions = vec![
        "Extract information accurately even from unstructured text".to_string(),
        "Use 'Unknown' for missing information".to_string(),
        "Ensure the output is valid JSON despite model limitations".to_string(),
        "Focus on the most relevant keywords (max 5)".to_string(),
    ];

    // Example research paper text (could be from various sources)
    let paper_text = "
        Title: Deep Learning Approaches for Natural Language Understanding in Conversational AI
        
        Authors: Dr. Sarah Chen (Stanford University), Prof. Michael Rodriguez (MIT), 
        Dr. Aisha Patel (Google Research), James Wilson (OpenAI)
        
        Published: 2024, Journal of Artificial Intelligence Research
        
        Abstract: This paper presents novel deep learning architectures for improving 
        natural language understanding in conversational AI systems. We introduce a 
        transformer-based approach that combines attention mechanisms with memory networks 
        to achieve state-of-the-art performance on dialogue understanding tasks. Our method 
        shows significant improvements over existing baselines on multiple benchmarks, 
        including a 15% increase in intent classification accuracy and 12% improvement 
        in entity extraction precision.
        
        Keywords: natural language processing, conversational AI, deep learning, 
        transformer networks, attention mechanisms
        
        This work was peer-reviewed and accepted at the top-tier JAIR conference.
    ";

    println!("Input text:");
    println!("{}", paper_text);
    println!();

    // Display the generated system prompt
    println!("Generated System Prompt:");
    println!("{}", task.get_system_prompt());
    println!();

    // Note: This would require actual API credentials to work
    println!("Setting up async force LLM call (for reasoning models without JSON mode):");

    let llm = OpenAILLM::new(
        &std::env::var("SECRETARY_OPENAI_API_BASE").unwrap(),
        &std::env::var("SECRETARY_OPENAI_API_KEY").unwrap(),
        &std::env::var("SECRETARY_OPENAI_MODEL").unwrap(), // Could be o1-preview, deepseek-reasoner, etc.
    )?;

    println!("Making async force request to LLM (bypassing JSON mode requirement)...");

    // Use async_force_generate_data instead of async_generate_data
    // This method works with reasoning models that don't support JSON mode
    let result: ResearchPaperExtraction = llm
        .async_force_generate_data(&task, paper_text, &additional_instructions)
        .await?;

    println!("Generated Data Structure: {:#?}", result);

    Ok(())
}
