use secretary::llm_providers::openai::OpenAILLM;
use secretary::traits::{AsyncGenerateData, Task};
use serde::{Deserialize, Serialize};
use tokio;

/// Example data structure for extracting product information
#[derive(Task, Serialize, Deserialize, Debug)]
struct ProductExtraction {
    /// Product data fields with specific extraction instructions
    #[task(instruction = "Extract the product name or title")]
    pub name: String,

    #[task(instruction = "Extract the price as a number (without currency symbols)")]
    pub price: f64,

    #[task(instruction = "Extract the product category or type")]
    pub category: String,

    #[task(instruction = "Extract the brand name if mentioned")]
    pub brand: Option<String>,

    #[task(instruction = "Extract key features or description")]
    pub description: String,

    #[task(instruction = "Determine if the product is in stock (true/false)")]
    pub in_stock: bool,
}

/// Async example demonstrating the Task derive macro
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    println!("Secretary Async Example - Product Information Extraction");
    println!("{}", "=".repeat(60));

    // Create a task instance
    let task = ProductExtraction::new();

    // Additional instructions for the LLM
    let additional_instructions = vec![
        "Be precise with numerical values".to_string(),
        "Ensure boolean values are accurate".to_string(),
    ];

    // Example product description text
    let product_text = "
        Apple MacBook Pro 16-inch - $2,499
        
        The latest MacBook Pro features the powerful M3 Pro chip, 
        16GB unified memory, and 512GB SSD storage. Perfect for 
        professional video editing and software development.
        
        Category: Laptop Computer
        Status: In Stock
        Brand: Apple
    ";

    println!("Input text:");
    println!("{}", product_text);
    println!();

    // Display the generated system prompt
    println!("Generated System Prompt:");
    println!(
        "{:#?}",
        task.get_system_prompts_for_distributed_generation()
    );
    println!();

    // Note: This would require actual API credentials to work
    // For demonstration, we'll show how to set up the async call
    println!("Setting up async LLM call (requires API credentials):");

    let llm = OpenAILLM::new(
        &std::env::var("SECRETARY_OPENAI_API_BASE").unwrap(),
        &std::env::var("SECRETARY_OPENAI_API_KEY").unwrap(),
        &std::env::var("SECRETARY_OPENAI_MODEL").unwrap(),
    )?;

    println!("Making async request to LLM...");
    let result: ProductExtraction = llm
        .async_fields_generate_data(&task, product_text, &additional_instructions)
        .await?;
    println!("Generated Data Structure: {:#?}", result);

    println!();
    println!("Example completed successfully!");

    Ok(())
}
