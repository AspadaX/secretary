use secretary::{llm::OpenAILLM, prompt::Prompt, traits::GenerateJSON};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sentiment {
    sentiment: String,
}

fn main() {
    let llm = OpenAILLM::new("your_api_base_url", "your_api_key", "your_model").unwrap();
    let data_structure = Sentiment {
        sentiment: String::from("rate the text in terms of their sentiments. it can be: high, low or mid.")
    };
    let mut prompt = Prompt::new(data_structure, vec![]);
    // You need to add `async_openai` to assign the role here
    prompt.push(async_openai::types::Role::User, "This is unacceptable!").unwrap();
    
    let result: String = llm.generate_json_with_context(prompt).unwrap();
    
    println!("{}", result);
}