use secretary::llm_providers::openai::OpenAILLM;
use secretary::traits::{GenerateData, Task};
use serde::{Deserialize, Serialize};

#[derive(Task, Serialize, Deserialize, Debug)]
struct Info {
    #[task(instruction = "Extract the person's email address if mentioned")]
    pub email: Option<String>,

    #[task(instruction = "Extract the person's occupation or job title")]
    pub occupation: String,
}

/// Example data structure using the Task derive macro
#[derive(Task, Serialize, Deserialize, Debug)]
struct PersonExtraction {
    /// Data fields with extraction instructions
    #[task(instruction = "Extract the person's full name from the text")]
    pub name: String,

    #[task(instruction = "Extract the person's age as a number")]
    pub age: u32,

    pub info: Info
}

/// Example showing how to use the derive macro
fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    // Create a new task instance
    let task = PersonExtraction::new();

    // Additional instructions for the LLM
    let additional_instructions = vec![
        "Focus on extracting accurate information".to_string(),
        "If information is not available, use appropriate defaults".to_string(),
    ];

    // Example text to extract from
    let text =
        "John Smith is a 30-year-old software engineer. You can reach him at john.smith@email.com";

    // Create LLM instance (you would use real API credentials)
    let llm = OpenAILLM::new(
        &std::env::var("SECRETARY_OPENAI_API_BASE").unwrap(),
        &std::env::var("SECRETARY_OPENAI_API_KEY").unwrap(),
        &std::env::var("SECRETARY_OPENAI_MODEL").unwrap(),
    )?;

    // Generate structured data using the task
    let result: PersonExtraction = llm.generate_data(&task, text, &additional_instructions)?;
    println!("Generated data: {:#?}", result);

    Ok(())
}
