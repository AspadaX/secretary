use serde::{Serialize, Deserialize};
use secretary::llm_providers::openai::OpenAILLM;
use secretary::traits::{GenerateData, Task};

/// Example data structure using the new Task derive macro
#[derive(Task, Serialize, Deserialize, Debug, Default)]
struct PersonExtraction {
    /// The context and additional instructions (required by Task trait)
    #[serde(skip)]
    pub context: secretary::MessageList,
    #[serde(skip)]
    pub additional_instructions: Vec<String>,
    
    /// Data fields with extraction instructions
    #[task(instruction = "Extract the person's full name from the text")]
    pub name: String,
    
    #[task(instruction = "Extract the person's age as a number")]
    pub age: u32,
    
    #[task(instruction = "Extract the person's email address if mentioned")]
    pub email: Option<String>,
    
    #[task(instruction = "Extract the person's occupation or job title")]
    pub occupation: String,
}

/// Example showing how to use the derive macro
fn main() -> anyhow::Result<()> {
    // Create a new task instance with additional instructions
    let task = PersonExtraction::new(vec![
        "Focus on extracting accurate information".to_string(),
        "If information is not available, use appropriate defaults".to_string(),
    ]);
    
    // Example text to extract from
    let text = "John Smith is a 30-year-old software engineer. You can reach him at john.smith@email.com";
    
    // Create LLM instance (you would use real API credentials)
    let llm = OpenAILLM::new(
        &std::env::var("SECRETARY_OPENAI_API_BASE").unwrap(),
        &std::env::var("SECRETARY_OPENAI_API_KEY").unwrap(),
        &std::env::var("SECRETARY_OPENAI_MODEL").unwrap(),
    )?;
    
    // Generate JSON using the task
    let result: PersonExtraction = llm.generate_data(&task, text)?;
    println!("Generated JSON: {:#?}", result);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_task_creation() {
        let task = PersonExtraction::new(vec!["test instruction".to_string()]);
        assert_eq!(task.additional_instructions.len(), 1);
        assert_eq!(task.additional_instructions[0], "test instruction");
    }
    
    #[test]
    fn test_system_prompt_generation() {
        let task = PersonExtraction::new(vec!["additional instruction".to_string()]);
        let prompt = task.get_system_prompt();
        
        // Check that the prompt contains the expected elements
        assert!(prompt.contains("json structure"));
        assert!(prompt.contains("Field instructions"));
        assert!(prompt.contains("Additional instructions"));
        assert!(prompt.contains("additional instruction"));
    }
}