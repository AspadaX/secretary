use reqwest::Client;
use serde_json::json;

use crate::{constants::OPENAI_CHAT_COMPLETION_ROUTE, traits::{AsyncGenerateData, GenerateData, IsLLM, LLM}};

/// Represents a Large Language Model (LLM) that is compatible with OpenAI API.
/// An LLM is the primary tool we use to convert unstructured data into structured data.
#[derive(Debug, Clone)]
pub struct OpenAILLM {
    model: String,
    api_key: String,
    api_base: String,
    client: Client,
}

impl OpenAILLM {
    /// Creates a new instance of the LLM struct.
    ///
    /// # Arguments
    ///
    /// * `api_base` - A string slice that holds the base URL for the OpenAI API.
    /// * `api_key` - A string slice that holds the API key for authenticating with the OpenAI API.
    /// * `model` - A string slice that specifies the model to be used by the LLM.
    ///
    /// # Returns
    ///
    /// * `Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>>` - On success, returns an instance of the LLM struct. On failure, returns an Box<dyn std::error::Error + Send + Sync + 'static>.
    pub fn new(api_base: &str, api_key: &str, model: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let client: Client = Client::new();

        Ok(Self {
            model: model.to_string(),
            api_base: api_base.to_string(),
            api_key: api_key.to_string(),
            client,
        })
    }
}

impl IsLLM for OpenAILLM {
    fn send_message(&self, message: crate::message::Message) -> Result<String, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let request = self.access_client()
            .post(self.api_base + OPENAI_CHAT_COMPLETION_ROUTE)
            .body(
                json!(
                    {
                        "model": self.model,
                        "messages": [
                            message.to
                        ]
                    }
                )
            )
            .send()
            .await?
    }
    
    fn access_client(&self) -> &Client {
        &self.client
    }

    fn access_model(&self) -> &str {
        &self.model
    }
}

impl GenerateData for OpenAILLM {}

impl AsyncGenerateData for OpenAILLM {}
