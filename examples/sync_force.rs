use secretary::Task;
use secretary::llm_providers::openai::OpenAILLM;
use secretary::traits::GenerateData;
use serde::{Deserialize, Serialize};

/// Example data structure for extracting financial report information
/// This example demonstrates force generation for LLMs without JSON mode support
#[derive(Task, Serialize, Deserialize, Debug)]
struct FinancialReportExtraction {
    /// Financial report data fields with specific extraction instructions
    #[task(instruction = "Extract the company name")]
    pub company_name: String,

    #[task(instruction = "Extract the reporting quarter (e.g., Q1, Q2, Q3, Q4)")]
    pub quarter: String,

    #[task(instruction = "Extract the fiscal year as a number")]
    pub fiscal_year: u32,

    #[task(instruction = "Extract the total revenue as a number (in millions)")]
    pub revenue_millions: f64,

    #[task(instruction = "Extract the net income as a number (in millions)")]
    pub net_income_millions: f64,

    #[task(instruction = "Extract the earnings per share (EPS) as a number")]
    pub eps: f64,

    #[task(instruction = "Extract key business highlights or achievements")]
    pub highlights: Vec<String>,

    #[task(instruction = "Determine if the company met analyst expectations (true/false)")]
    pub met_expectations: bool,

    #[task(instruction = "Extract the CEO or key executive's name if mentioned")]
    pub ceo_name: Option<String>,
}

/// Sync force example demonstrating JSON parsing for reasoning models
/// This example shows how to use force_generate_data for models like o1 and deepseek
/// that don't have built-in JSON mode support but can still generate structured data
fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    println!("Secretary Sync Force Example - Financial Report Extraction");
    println!("{}", "=".repeat(62));
    println!("This example demonstrates force JSON parsing for reasoning models");
    println!("like o1 and deepseek that don't support native JSON mode.\n");

    // Create a task instance
    let task = FinancialReportExtraction::new();

    // Additional instructions for the LLM
    let additional_instructions = vec![
        "Extract numerical values accurately without currency symbols".to_string(),
        "Use 'Unknown' for missing information".to_string(),
        "Ensure the output is valid JSON despite model limitations".to_string(),
        "Focus on the most significant highlights (max 3)".to_string(),
    ];

    // Example financial report text
    let report_text = "
        APPLE INC. REPORTS FOURTH QUARTER 2024 RESULTS
        
        CUPERTINO, California — Apple Inc. (NASDAQ: AAPL) today announced financial 
        results for its fiscal 2024 fourth quarter ended September 28, 2024.
        
        Fourth Quarter 2024 Financial Results:
        • Total net sales: $94.9 billion, up 6% year-over-year
        • Net income: $22.9 billion, or $1.46 per diluted share
        • iPhone revenue: $46.2 billion, up 6% year-over-year
        • Services revenue: $24.2 billion, up 12% year-over-year
        
        \"We are pleased with our strong fourth quarter results, which exceeded 
        analyst expectations across all major product categories,\" said Tim Cook, 
        Apple's CEO. \"Our continued innovation in AI and services drove exceptional 
        growth this quarter.\"
        
        Key Highlights:
        - Record Services revenue of $24.2 billion
        - Strong iPhone 15 adoption with Pro models leading sales
        - Successful launch of Apple Intelligence features
        - Expansion into new international markets
        
        The company exceeded Wall Street expectations, with analysts predicting 
        earnings of $1.40 per share and revenue of $94.5 billion.
    ";

    println!("Input text:");
    println!("{}", report_text);
    println!();

    // Display the generated system prompt
    println!("Generated System Prompt:");
    println!("{}", task.get_system_prompt());
    println!();

    // Note: This would require actual API credentials to work
    println!("Setting up sync force LLM call (for reasoning models without JSON mode):");

    let llm = OpenAILLM::new(
        &std::env::var("SECRETARY_OPENAI_API_BASE").unwrap(),
        &std::env::var("SECRETARY_OPENAI_API_KEY").unwrap(),
        &std::env::var("SECRETARY_OPENAI_MODEL").unwrap(), // Could be o1-preview, deepseek-reasoner, etc.
    )?;

    println!("Making sync force request to LLM (bypassing JSON mode requirement)...");

    // Use force_generate_data instead of generate_data
    // This method works with reasoning models that don't support JSON mode
    let result: FinancialReportExtraction =
        llm.force_generate_data(&task, report_text, &additional_instructions)?;

    println!("Generated Data Structure: {:#?}", result);

    Ok(())
}
