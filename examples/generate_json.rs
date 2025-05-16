use secretary::{openai::OpenAILLM, prompt::Prompt, traits::GenerateJSON};
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
    let prompt = Prompt::new(data_structure, vec![]);
    
    let result: String = llm.generate_json(&prompt, "This is unacceptable!").unwrap();
    
    println!("{}", result);
}