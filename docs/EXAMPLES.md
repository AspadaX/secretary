# Secretary Examples

This document provides comprehensive examples of using the Secretary library for various use cases.

## Basic Examples

### Simple JSON Generation

Extract a sentiment classification from text:

```rust
use secretary::{openai::OpenAILLM, tasks::basic_task::BasicTask, traits::GenerateJSON};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sentiment {
    sentiment: String,
}

impl Default for Sentiment {
    fn default() -> Self {
        Sentiment {
            sentiment: String::from(
                "rate the text in terms of their sentiments. it can be: high, low or mid.",
            ),
        }
    }
}

fn main() {
    // Initialize the OpenAI LLM client
    let llm = OpenAILLM::new(
        &std::env::var("SECRETARY_OPENAI_API_BASE").unwrap(),
        &std::env::var("SECRETARY_OPENAI_API_KEY").unwrap(),
        &std::env::var("SECRETARY_OPENAI_MODEL").unwrap(),
    )
    .unwrap();

    // Create task with schema and instructions
    let task = BasicTask::new(
        Sentiment::default(),
        vec![
            "Consider the context when determining sentiment.".to_string(),
            "Pay attention to intensifiers and negations that might change sentiment.".to_string(),
        ],
    );

    // Generate JSON from text
    let result: String = llm.generate_json(&task, "This is unacceptable!").unwrap();
    println!("{}", result);
    // Output: {"sentiment":"low"}
}
```

### Extracting Multiple Fields

Extract product information from a description:

```rust
use secretary::{openai::OpenAILLM, tasks::basic_task::BasicTask, traits::GenerateJSON};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Product {
    name: String,
    category: String,
    price_range: String,
    features: Vec<String>,
    target_audience: Option<String>,
}

impl Default for Product {
    fn default() -> Self {
        Self {
            name: "Product name".to_string(),
            category: "Product category (electronics, clothing, etc.)".to_string(),
            price_range: "Approximate price range (budget, mid-range, premium)".to_string(),
            features: vec!["Key product features mentioned in the text".to_string()],
            target_audience: Some("The intended users of this product".to_string()),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let llm = OpenAILLM::new(
        &std::env::var("SECRETARY_OPENAI_API_BASE").unwrap(),
        &std::env::var("SECRETARY_OPENAI_API_KEY").unwrap(),
        &std::env::var("SECRETARY_OPENAI_MODEL").unwrap(),
    )?;
    
    let task = BasicTask::new(
        Product::default(),
        vec![
            "Extract all product details mentioned in the text".to_string(),
            "Infer the price range and category if not explicitly stated".to_string(),
        ],
    );
    
    let description = "The new XPS 13 laptop features a 13-inch 4K display, \
                      16GB of RAM, and comes with the latest 12th Gen Intel processor. \
                      Perfect for professionals and students who need performance on the go.";
    
    let json_result = llm.generate_json(&task, description)?;
    println!("{}", json_result);
    
    // Parse the result into our struct
    let product: Product = serde_json::from_str(&json_result)?;
    println!("Product name: {}", product.name);
    println!("Features:");
    for feature in product.features {
        println!(" - {}", feature);
    }
    
    Ok(())
}
```

## Multi-Turn Conversations

### Basic Conversation Flow

Maintain context across multiple exchanges:

```rust
use secretary::{
    message_list::Role, 
    openai::OpenAILLM, 
    tasks::basic_task::BasicTask, 
    traits::{Context, GenerateJSON},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    item: String,
    reason: String,
}

impl Default for Recommendation {
    fn default() -> Self {
        Recommendation {
            item: String::from("Recommended item name"),
            reason: String::from("Reason for the recommendation"),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let llm = OpenAILLM::new(
        &std::env::var("SECRETARY_OPENAI_API_BASE").unwrap(),
        &std::env::var("SECRETARY_OPENAI_API_KEY").unwrap(),
        &std::env::var("SECRETARY_OPENAI_MODEL").unwrap(),
    )?;

    let mut task = BasicTask::new(
        Recommendation::default(), 
        vec![
            "Provide personalized recommendations based on user preferences".to_string(),
            "Consider all previous messages in the conversation".to_string(),
        ],
    );

    // Start conversation
    task.push(Role::User, "I'm looking for a new book to read.")?;
    
    // Get first recommendation
    let response1 = llm.generate_json_with_context(&task)?;
    println!("First recommendation: {}", response1);
    
    // Add response to conversation context
    task.push(Role::Assistant, &response1)?;
    
    // Continue conversation
    task.push(Role::User, "I prefer science fiction with female protagonists.")?;
    
    // Get updated recommendation based on new information
    let response2 = llm.generate_json_with_context(&task)?;
    println!("Updated recommendation: {}", response2);
    
    Ok(())
}
```

## Contextual Tasks

### Progressive Data Building

Build a complex data structure through conversation:

```rust
use secretary::{
    message_list::Role,
    openai::OpenAILLM,
    tasks::contextual_task::ContextualTask,
    traits::{Context, GenerateJSON},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Resume {
    name: String,
    education: Vec<String>,
    work_experience: Vec<String>,
    skills: Vec<String>,
    certifications: Option<Vec<String>>,
}

impl Default for Resume {
    fn default() -> Self {
        Self {
            name: "Full name of the person".to_string(),
            education: vec!["Education history with degrees, institutions, and years".to_string()],
            work_experience: vec!["Work history with company, position, and dates".to_string()],
            skills: vec!["Technical and soft skills mentioned".to_string()],
            certifications: Some(vec!["Professional certifications if any".to_string()]),
        }
    }
}

fn simulate_conversation() -> anyhow::Result<()> {
    let llm = OpenAILLM::new(
        &std::env::var("SECRETARY_OPENAI_API_BASE").unwrap(),
        &std::env::var("SECRETARY_OPENAI_API_KEY").unwrap(),
        &std::env::var("SECRETARY_OPENAI_MODEL").unwrap(),
    )?;
    
    let mut task = ContextualTask::new(
        Resume::default(),
        vec![
            "Extract resume information progressively through conversation".to_string(),
            "Ask focused questions about missing information".to_string(),
            "Update the resume data structure as new information is provided".to_string(),
        ],
    );
    
    // Simulate a conversation to build a resume
    let conversation = [
        "I'm John Smith, a software engineer with 5 years of experience.",
        "I graduated from MIT with a computer science degree in 2018.",
        "I've worked at Google as a Senior Developer and at Tesla as a Software Engineer.",
        "My skills include Python, Rust, and machine learning.",
        "I'm certified in AWS and have a Project Management Professional certification."
    ];
    
    for (i, message) in conversation.iter().enumerate() {
        println!("\n--- Turn {} ---", i+1);
        println!("User: {}", message);
        
        // Add user message to context
        task.push(Role::User, message)?;
        
        // Generate response
        let response = llm.generate_json_with_context(&task)?;
        
        // Parse the response
        let json_value: serde_json::Value = serde_json::from_str(&response)?;
        
        // Display reasoning and content (question)
        if let Some(reasoning) = json_value.get("reasoning").and_then(|v| v.as_str()) {
            println!("\nAI Reasoning: {}", reasoning);
        }
        
        if let Some(content) = json_value.get("content").and_then(|v| v.as_str()) {
            println!("AI: {}", content);
        }
        
        // Display current resume state
        if let Some(data) = json_value.get("data_structure") {
            println!("\nCurrent Resume State:");
            println!("{}", serde_json::to_string_pretty(data)?);
        }
        
        // Add response to context
        task.push(Role::Assistant, &response)?;
    }
    
    Ok(())
}

fn main() {
    simulate_conversation().unwrap();
}
```

## Advanced Examples

### Asynchronous Processing

Process requests asynchronously:

```rust
use secretary::{
    openai::OpenAILLM, 
    tasks::basic_task::BasicTask, 
    traits::AsyncGenerateJSON,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct QueryAnalysis {
    intent: String,
    entities: Vec<String>,
    sentiment: String,
}

impl Default for QueryAnalysis {
    fn default() -> Self {
        Self {
            intent: "The user's primary intention".to_string(),
            entities: vec!["Key entities mentioned in the query".to_string()],
            sentiment: "Emotional tone: positive, negative, or neutral".to_string(),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let llm = OpenAILLM::new(
        &std::env::var("SECRETARY_OPENAI_API_BASE").unwrap(),
        &std::env::var("SECRETARY_OPENAI_API_KEY").unwrap(),
        &std::env::var("SECRETARY_OPENAI_MODEL").unwrap(),
    )?;
    
    let task = BasicTask::new(
        QueryAnalysis::default(),
        vec![
            "Analyze user queries for intent, entities, and sentiment".to_string(),
            "Focus on identifying actionable intents".to_string(),
        ],
    );
    
    let queries = vec![
        "Can you find me flights to New York for next weekend?",
        "I'm extremely disappointed with your customer service",
        "What's the weather forecast for Chicago tomorrow?",
    ];
    
    let mut handles = Vec::new();
    
    // Process queries concurrently
    for query in queries {
        let llm_clone = llm.clone();
        let task_clone = task.clone();
        let query_clone = query.to_string();
        
        let handle = tokio::spawn(async move {
            let result = llm_clone.async_generate_json(&task_clone, &query_clone).await?;
            Ok::<(String, String), anyhow::Error>((query_clone, result))
        });
        
        handles.push(handle);
    }
    
    // Collect and display results
    for handle in handles {
        let (query, result) = handle.await??;
        println!("Query: {}", query);
        println!("Analysis: {}", result);
        println!();
    }
    
    Ok(())
}
```

### Custom Data Processing

Process structured data after extraction:

```rust
use secretary::{openai::OpenAILLM, tasks::basic_task::BasicTask, traits::GenerateJSON};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct MeetingNotes {
    title: String,
    date: String,
    participants: Vec<String>,
    action_items: Vec<ActionItem>,
    key_points: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ActionItem {
    description: String,
    assignee: String,
    due_date: Option<String>,
}

impl Default for MeetingNotes {
    fn default() -> Self {
        Self {
            title: "Meeting title or topic".to_string(),
            date: "Date when the meeting occurred".to_string(),
            participants: vec!["List of people who attended the meeting".to_string()],
            action_items: vec![ActionItem {
                description: "Task to be completed".to_string(),
                assignee: "Person responsible for the task".to_string(),
                due_date: Some("When the task should be completed".to_string()),
            }],
            key_points: vec!["Important points discussed in the meeting".to_string()],
        }
    }
}

fn process_meeting_notes(notes: &str) -> anyhow::Result<()> {
    let llm = OpenAILLM::new(
        &std::env::var("SECRETARY_OPENAI_API_BASE").unwrap(),
        &std::env::var("SECRETARY_OPENAI_API_KEY").unwrap(),
        &std::env::var("SECRETARY_OPENAI_MODEL").unwrap(),
    )?;
    
    let task = BasicTask::new(
        MeetingNotes::default(),
        vec![
            "Extract structured information from meeting notes".to_string(),
            "Identify all action items and their assignees".to_string(),
            "Extract key discussion points".to_string(),
        ],
    );
    
    let json_result = llm.generate_json(&task, notes)?;
    
    // Parse the result
    let meeting: MeetingNotes = serde_json::from_str(&json_result)?;
    
    // Process the structured data
    println!("Meeting: {} ({})", meeting.title, meeting.date);
    println!("Participants: {}", meeting.participants.join(", "));
    
    println!("\nKey Points:");
    for (i, point) in meeting.key_points.iter().enumerate() {
        println!("{}. {}", i+1, point);
    }
    
    // Organize action items by assignee
    let mut tasks_by_assignee: HashMap<String, Vec<&ActionItem>> = HashMap::new();
    for item in &meeting.action_items {
        tasks_by_assignee
            .entry(item.assignee.clone())
            .or_insert_with(Vec::new)
            .push(item);
    }
    
    println!("\nAction Items by Person:");
    for (person, tasks) in tasks_by_assignee {
        println!("\n{}", person);
        for (i, task) in tasks.iter().enumerate() {
            let due = task.due_date.as_deref().unwrap_or("No due date");
            println!("{}. {} (Due: {})", i+1, task.description, due);
        }
    }
    
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let meeting_notes = "
        Product Planning Meeting - June 15, 2023
        
        Attendees: Alice (PM), Bob (Engineering), Carol (Design), David (Marketing)
        
        We discussed the Q3 roadmap for our mobile app. Alice presented the user feedback
        from our beta testers. Key points:
        - Users want better notification management
        - The checkout flow has too many steps
        - Performance issues on older Android devices
        
        Action items:
        - Bob will investigate the performance issues and propose solutions by June 30
        - Carol will redesign the checkout flow by June 22
        - David will create a marketing plan for the Q3 release by next Friday
        - Alice will prioritize the notification features for the next sprint
        
        Next meeting scheduled for June 29.
    ";
    
    process_meeting_notes(meeting_notes)?;
    Ok(())
}
```

## Performance Tips

- Provide clear, specific instructions in your task
- Use descriptive field names and annotations in your schema
- Start with simple schemas and add complexity gradually
- For large data structures, break the extraction into multiple steps
- Store examples of good extractions to improve your schemas

## Error Handling

```rust
use secretary::{openai::OpenAILLM, tasks::basic_task::BasicTask, traits::GenerateJSON};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Address {
    street: String,
    city: String,
    state: String,
    zip: String,
}

impl Default for Address {
    fn default() -> Self {
        Self {
            street: "Street address including house/apt number".to_string(),
            city: "City name".to_string(),
            state: "State name or abbreviation".to_string(),
            zip: "ZIP or postal code".to_string(),
        }
    }
}

fn extract_address(text: &str) -> Result<Address, String> {
    // Initialize the LLM client
    let llm = match OpenAILLM::new(
        &std::env::var("SECRETARY_OPENAI_API_BASE").unwrap_or("https://api.openai.com/v1".to_string()),
        &std::env::var("SECRETARY_OPENAI_API_KEY").map_err(|e| format!("API key error: {}", e))?,
        &std::env::var("SECRETARY_OPENAI_MODEL").unwrap_or("gpt-4o-mini".to_string()),
    ) {
        Ok(client) => client,
        Err(e) => return Err(format!("Failed to initialize LLM client: {}", e)),
    };
    
    // Create the extraction task
    let task = BasicTask::new(
        Address::default(),
        vec![
            "Extract address information from the text".to_string(),
            "If any field is missing, return null for that field".to_string(),
        ],
    );
    
    // Generate JSON and handle potential errors
    let json_result = match llm.generate_json(&task, text) {
        Ok(result) => result,
        Err(e) => return Err(format!("Generation error: {}", e)),
    };
    
    // Parse the JSON result
    match serde_json::from_str::<Address>(&json_result) {
        Ok(address) => Ok(address),
        Err(e) => Err(format!("Failed to parse address JSON: {}", e)),
    }
}

fn main() {
    let input = "Please ship to: 123 Main St, Springfield, IL 62701";
    
    match extract_address(input) {
        Ok(address) => {
            println!("Successfully extracted address:");
            println!("Street: {}", address.street);
            println!("City: {}", address.city);
            println!("State: {}", address.state);
            println!("ZIP: {}", address.zip);
        },
        Err(e) => {
            eprintln!("Error extracting address: {}", e);
        }
    }
}
```