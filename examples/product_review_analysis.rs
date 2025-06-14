//! # Secretary - Product Review Analysis Example
//! 
//! This example demonstrates how to use the Secretary library to analyze product reviews
//! and extract structured information such as sentiment, key points, and recommendations.
//!
//! ## What This Example Shows
//!
//! This example demonstrates:
//! 1. Creating a schema for product review analysis
//! 2. Extracting multiple fields of different types (strings, arrays, enums)
//! 3. Using the extracted data to generate insights
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
//! cargo run --example product_review_analysis
//! ```

use secretary::{openai::OpenAILLM, tasks::basic_task::BasicTask, traits::{DataModel, GenerateJSON}};
use serde::{Deserialize, Serialize};

/// Data structure representing the analysis of a product review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewAnalysis {
    /// Overall sentiment of the review (positive, negative, or mixed)
    sentiment: String,
    
    /// Rating on a scale of 1-5 inferred from the review
    rating: u8,
    
    /// Key positive points mentioned in the review
    pros: Vec<String>,
    
    /// Key negative points mentioned in the review
    cons: Vec<String>,
    
    /// Features mentioned in the review
    mentioned_features: Vec<String>,
    
    /// Recommendations from the reviewer
    recommendations: Option<String>,
    
    /// Whether the reviewer would buy the product again
    would_buy_again: Option<bool>,
}

impl DataModel for ReviewAnalysis {
    fn provide_data_model_instructions() -> Self {
        Self {
            sentiment: "The overall sentiment of the review (positive, negative, or mixed)".to_string(),
            rating: 0, // Will be populated with a value between 1-5
            pros: vec!["Positive aspects mentioned in the review".to_string()],
            cons: vec!["Negative aspects mentioned in the review".to_string()],
            mentioned_features: vec!["Product features mentioned in the review".to_string()],
            recommendations: Some("Any recommendations the reviewer gives for improvement".to_string()),
            would_buy_again: None, // This will be populated if the reviewer indicates purchase intent
        }
    }
}

fn main() {
    // Initialize the OpenAI LLM client
    let llm = OpenAILLM::new(
        &std::env::var("SECRETARY_OPENAI_API_BASE").unwrap_or("https://api.openai.com/v1".to_string()),
        &std::env::var("SECRETARY_OPENAI_API_KEY").unwrap(),
        &std::env::var("SECRETARY_OPENAI_MODEL").unwrap_or("gpt-4o-mini".to_string()),
    )
    .unwrap();

    // Create a task for review analysis
    let task = BasicTask::new::<ReviewAnalysis>(
        vec![
            "Extract key information from product reviews".to_string(),
            "For 'rating', infer a 1-5 star rating based on the sentiment and content".to_string(),
            "For 'pros' and 'cons', list specific points, not general statements".to_string(),
            "For 'would_buy_again', only include if the reviewer explicitly mentions repurchase intent".to_string(),
        ],
    );

    // Sample product reviews to analyze
    let reviews = vec![
        "I've been using this phone for about a month now, and I'm really impressed with the battery life and camera quality. The screen is gorgeous and the processor is lightning fast. However, I find the new gesture navigation confusing and the price is a bit high for what you get. I'd recommend waiting for a sale, but overall it's a solid device. Would probably buy from this company again.",
        
        "Terrible experience with this blender. It worked fine for the first week, then started making a loud noise. The build quality is cheap and the warranty process is a nightmare. The only positive is that it looks nice on the counter. Save your money and buy something better. I regret this purchase completely.",
        
        "This desk chair is just okay. The cushioning is comfortable and it has good lumbar support, but the armrests are wobbly and the height adjustment keeps sinking throughout the day. Assembly was straightforward, taking about 30 minutes. For the price, it's decent, but I wish I had spent a bit more for better quality. If they improved the hydraulics and armrests, it would be a great chair."
    ];

    println!("🔍 Product Review Analysis Example");
    println!("=================================\n");

    // Process each review
    for (i, review) in reviews.iter().enumerate() {
        println!("📝 Review #{}: ", i+1);
        println!("{}\n", review);

        // Generate JSON analysis of the review
        let result = llm.generate_json(&task, review).unwrap();
        
        // Parse the JSON into our ReviewAnalysis structure
        let analysis: ReviewAnalysis = serde_json::from_str(&result).unwrap();
        
        // Display the analysis
        println!("✨ Analysis Results:");
        println!("Sentiment: {}", analysis.sentiment);
        println!("Inferred Rating: {} out of 5", analysis.rating);
        
        println!("\n👍 Pros:");
        for pro in analysis.pros {
            println!("  • {}", pro);
        }
        
        println!("\n👎 Cons:");
        for con in analysis.cons {
            println!("  • {}", con);
        }
        
        println!("\n🔧 Mentioned Features:");
        for feature in analysis.mentioned_features {
            println!("  • {}", feature);
        }
        
        if let Some(recommendations) = analysis.recommendations {
            println!("\n💡 Recommendations:");
            println!("  {}", recommendations);
        }
        
        if let Some(would_buy_again) = analysis.would_buy_again {
            println!("\n🛒 Would Buy Again: {}", if would_buy_again { "Yes" } else { "No" });
        } else {
            println!("\n🛒 Would Buy Again: Not specified");
        }
        
        println!("\n=================================\n");
    }
}