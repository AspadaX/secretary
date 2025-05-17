//! # Secretary - Data Analysis with ContextualTask Example
//! 
//! This example demonstrates using ContextualTask for analytical tasks by 
//! processing a dataset and generating structured analysis with reasoning.
//! The AI analyzes the data, shows its reasoning process, and extracts insights 
//! into a structured format.
//!
//! ## Analytical Capabilities of ContextualTask
//!
//! ContextualTask is well-suited for analytical tasks because it:
//! 1. **Shows reasoning**: Documents the analytical process transparently
//! 2. **Builds insights progressively**: Accumulates key points in the notes field
//! 3. **Structures findings**: Organizes analysis into a consistent data schema
//! 4. **Highlights uncertainties**: Can indicate confidence levels and data gaps
//!
//! ## Analysis Process Flow
//!
//! 1. Define an analysis schema with the insights you want to extract
//! 2. Provide the dataset and analysis instructions
//! 3. Examine the AI's reasoning, notes, and structured findings
//! 4. Use the analysis to make data-driven decisions
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
//! cargo run --example contextual_prompt_analysis
//! ```

use anyhow::Result;
use secretary::{
    message_list::Role,
    openai::OpenAILLM,
    tasks::contextual_task::ContextualTask,
    traits::{Context, GenerateJSON},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Data analysis schema for sales performance insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesAnalysis {
    /// Overall sales trend (increasing, decreasing, stable)
    trend: String,
    
    /// Top performing product categories
    top_categories: Vec<String>,
    
    /// Underperforming product categories
    underperforming_categories: Vec<String>,
    
    /// Seasonal patterns identified in the data
    seasonal_patterns: Option<Vec<String>>,
    
    /// Key insights about customer behavior
    customer_insights: Vec<String>,
    
    /// Actionable recommendations based on the analysis
    recommendations: Vec<String>,
    
    /// Data quality issues or limitations
    data_limitations: Option<Vec<String>>,
    
    /// Confidence level in analysis (high, medium, low)
    confidence: String,
}

impl Default for SalesAnalysis {
    fn default() -> Self {
        SalesAnalysis {
            trend: String::from("increasing, decreasing, or stable"),
            top_categories: Vec::new(),
            underperforming_categories: Vec::new(),
            seasonal_patterns: None,
            customer_insights: Vec::new(),
            recommendations: Vec::new(),
            data_limitations: None,
            confidence: String::from("high, medium, or low"),
        }
    }
}

/// Sample dataset of quarterly sales figures
fn get_sample_sales_data() -> String {
    // This would typically come from a database, CSV file, or API
    r#"
    Quarterly Sales Data (2022-2023)

    Q1 2022:
    - Electronics: $245,000
    - Home & Kitchen: $189,500
    - Clothing: $210,300
    - Books: $95,700
    - Toys: $78,200

    Q2 2022:
    - Electronics: $198,400
    - Home & Kitchen: $204,600
    - Clothing: $178,500
    - Books: $82,100
    - Toys: $67,800

    Q3 2022:
    - Electronics: $228,900
    - Home & Kitchen: $256,700
    - Clothing: $163,200
    - Books: $105,300
    - Toys: $142,600

    Q4 2022:
    - Electronics: $389,700
    - Home & Kitchen: $226,400
    - Clothing: $197,100
    - Books: $127,800
    - Toys: $231,500

    Q1 2023:
    - Electronics: $267,800
    - Home & Kitchen: $192,300
    - Clothing: $205,600
    - Books: $89,400
    - Toys: $72,100

    Q2 2023:
    - Electronics: $212,500
    - Home & Kitchen: $218,700
    - Clothing: $182,900
    - Books: $79,300
    - Toys: $71,200

    Customer Demographic Info:
    - 35% of electronics purchases from 18-34 age group
    - 58% of toy purchases during holiday season (Q4)
    - Home & Kitchen showing 15% increase in repeat customers
    - Clothing sales 40% higher on weekends
    - Book sales 30% higher during promotional periods
    "#.to_string()
}

fn main() -> Result<()> {
    println!("📊 Sales Data Analysis with ContextualTask");
    println!("This example demonstrates analytical reasoning and structured insight extraction.\n");
    
    // Initialize the OpenAI LLM client
    let llm = OpenAILLM::new(
        &std::env::var("SECRETARY_OPENAI_API_BASE").unwrap_or("https://api.openai.com/v1".to_string()),
        &std::env::var("SECRETARY_OPENAI_API_KEY").unwrap(),
        &std::env::var("SECRETARY_OPENAI_MODEL").unwrap_or("gpt-4o-mini".to_string()),
    )?;
    
    // Create a contextual prompt with our sales analysis schema
    let mut prompt = ContextualTask::new(
        SalesAnalysis::default(),
        vec![
            "Analyze the quarterly sales data to identify trends and patterns.".to_string(),
            "Show your mathematical reasoning in the reasoning field.".to_string(),
            "Calculate percentage changes between quarters to identify growth or decline.".to_string(),
            "Identify seasonal patterns by comparing the same quarters across years.".to_string(),
            "Use the notes field to track key observations as you analyze the data.".to_string(),
            "Provide actionable recommendations based on your findings.".to_string(),
            "Indicate any data limitations or areas where more information would be helpful.".to_string(),
        ],
    );
    
    // Get the sample sales data
    let sales_data = get_sample_sales_data();
    println!("Analyzing the following sales data:");
    println!("{}\n", sales_data);
    
    // Add the analysis request with sales data
    prompt.push(Role::User, &format!("Please analyze this sales data and provide insights:\n{}", sales_data))?;
    
    println!("Processing analysis...\n");
    
    // Generate a structured analysis with reasoning
    let json_response = llm.generate_json_with_context(&prompt)?;
    
    // Parse the JSON response
    let value: Value = serde_json::from_str(&json_response)?;
    
    // Display the AI's reasoning process
    if let Some(reasoning) = value.get("reasoning").and_then(|v| v.as_str()) {
        println!("## Analysis Reasoning Process\n");
        println!("{}\n", reasoning);
    }
    
    // Display the accumulated analytical notes
    if let Some(notes) = value.get("notes").and_then(|v| v.as_array()) {
        println!("## Key Analytical Observations\n");
        for (i, note) in notes.iter().enumerate() {
            if let Some(note_str) = note.as_str() {
                println!("{}. {}", i+1, note_str);
            }
        }
        println!();
    }
    
    // Display the structured analysis results
    if let Some(data) = value.get("data_structure") {
        println!("## Structured Analysis Results\n");
        
        if let Some(trend) = data.get("trend").and_then(|v| v.as_str()) {
            println!("📈 Overall Sales Trend: {}", trend);
        }
        
        if let Some(top) = data.get("top_categories").and_then(|v| v.as_array()) {
            println!("\n🏆 Top Performing Categories:");
            for cat in top {
                if let Some(cat_str) = cat.as_str() {
                    println!("  ✓ {}", cat_str);
                }
            }
        }
        
        if let Some(under) = data.get("underperforming_categories").and_then(|v| v.as_array()) {
            println!("\n⚠️ Underperforming Categories:");
            for cat in under {
                if let Some(cat_str) = cat.as_str() {
                    println!("  ✗ {}", cat_str);
                }
            }
        }
        
        if let Some(seasonal) = data.get("seasonal_patterns").and_then(|v| v.as_array()) {
            println!("\n🗓️ Seasonal Patterns:");
            for pattern in seasonal {
                if let Some(pattern_str) = pattern.as_str() {
                    println!("  • {}", pattern_str);
                }
            }
        }
        
        if let Some(insights) = data.get("customer_insights").and_then(|v| v.as_array()) {
            println!("\n👥 Customer Insights:");
            for insight in insights {
                if let Some(insight_str) = insight.as_str() {
                    println!("  • {}", insight_str);
                }
            }
        }
        
        if let Some(recs) = data.get("recommendations").and_then(|v| v.as_array()) {
            println!("\n💡 Recommendations:");
            for (i, rec) in recs.iter().enumerate() {
                if let Some(rec_str) = rec.as_str() {
                    println!("  {}. {}", i+1, rec_str);
                }
            }
        }
        
        if let Some(limitations) = data.get("data_limitations").and_then(|v| v.as_array()) {
            println!("\n⚖️ Data Limitations:");
            for limit in limitations {
                if let Some(limit_str) = limit.as_str() {
                    println!("  • {}", limit_str);
                }
            }
        }
        
        if let Some(confidence) = data.get("confidence").and_then(|v| v.as_str()) {
            println!("\n🎯 Confidence Level: {}", confidence);
        }
    }
    
    // Display a follow-up question if present
    if let Some(content) = value.get("content").and_then(|v| v.as_str()) {
        println!("\n## Follow-up Questions\n");
        println!("{}", content);
    }
    
    println!("\nThis example demonstrates how ContextualTask provides transparent reasoning and structured analysis,");
    println!("making it ideal for data analysis tasks where understanding the reasoning process is important.");
    
    Ok(())
}