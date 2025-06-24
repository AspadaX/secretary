use secretary::{AsyncGenerateData, llm_providers::openai::OpenAILLM, traits::Task};
use serde::{Deserialize, Serialize};
use tokio;

/// Example data structure for extracting product information
#[derive(Task, Serialize, Deserialize, Debug, Default)]
struct ProductExtraction {
    /// Required fields for Task trait
    #[serde(skip)]
    pub context: secretary::MessageList,
    #[serde(skip)]
    pub additional_instructions: Vec<String>,

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
async fn main() -> anyhow::Result<()> {
    println!("Secretary Async Example - Product Information Extraction");
    println!("{}", "=".repeat(60));

    // Create a task instance with additional instructions
    let mut task = ProductExtraction::new(vec![
        "Be precise with numerical values".to_string(),
        "Use 'Unknown' for missing information".to_string(),
        "Ensure boolean values are accurate".to_string(),
    ]);

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
    println!("{}", task.get_system_prompt());
    println!();

    // Demonstrate context management
    task.push(
        secretary::message_list::Role::User,
        "Please extract product information from the following text",
    )?;
    task.push(
        secretary::message_list::Role::Assistant,
        "I'll help you extract the product information accurately.",
    )?;

    println!("Context messages: {}", task.get_context().len());
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
    let result: ProductExtraction = llm.async_generate_data(&task, product_text).await?;
    println!("Generated Data Structure: {:#?}", result);

    println!();
    println!("Example completed successfully!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = ProductExtraction::new(vec!["test instruction".to_string()]);
        assert_eq!(task.get_additional_instructions().len(), 1);
        assert_eq!(task.get_additional_instructions()[0], "test instruction");
    }

    #[test]
    fn test_system_prompt_generation() {
        let task = ProductExtraction::new(vec![]);
        let prompt = task.get_system_prompt();

        // Check that the prompt contains expected elements
        assert!(prompt.contains("json structure"));
        assert!(prompt.contains("Field instructions"));
        assert!(prompt.contains("name"));
        assert!(prompt.contains("price"));
        assert!(prompt.contains("category"));
    }

    #[test]
    fn test_context_management() {
        let mut task = ProductExtraction::new(vec![]);

        // Test adding messages to context
        task.push(secretary::message_list::Role::User, "Test message")
            .unwrap();
        let context = task.get_context();

        // Should have system message + user message
        assert!(context.len() >= 2);
    }

    #[tokio::test]
    async fn test_async_compatibility() {
        // Test that our struct works in async context
        let task = ProductExtraction::new(vec!["async test".to_string()]);

        // Simulate async operation
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;

        assert_eq!(task.get_additional_instructions()[0], "async test");
    }
}
