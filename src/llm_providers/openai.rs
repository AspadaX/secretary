use serde_json::{Value, json};

use crate::{
    constants::OPENAI_CHAT_COMPLETION_ROUTE,
    message::Message,
    traits::{AsyncGenerateData, GenerateData, IsLLM},
};

/// Represents a Large Language Model (LLM) that is compatible with OpenAI API.
/// An LLM is the primary tool we use to convert unstructured data into structured data.
#[derive(Debug, Clone)]
pub struct OpenAILLM {
    model: String,
    api_key: String,
    api_base: String,
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
    pub fn new(
        api_base: &str,
        api_key: &str,
        model: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
        Ok(Self {
            model: model.to_string(),
            api_base: api_base.to_string(),
            api_key: api_key.to_string(),
        })
    }
}

impl IsLLM for OpenAILLM {
    fn get_authorization_credentials(&self) -> String {
        format!("Bearer {}", self.api_key)
    }

    fn get_model_ref(&self) -> &str {
        &self.model
    }

    fn get_chat_completion_request_url(&self) -> String {
        format!("{}{}", self.api_base, OPENAI_CHAT_COMPLETION_ROUTE)
    }

    fn get_reqeust_body(&self, message: Message, return_json: bool) -> Value {
        if return_json {
            return json!(
                {
                    "model": self.get_model_ref(),
                    "messages": [message],
                    "response_format": {"type": "json_object"}
                }
            );
        }

        return json!(
            {
                "model": self.get_model_ref(),
                "messages": [message],
            }
        );
    }
}

impl GenerateData for OpenAILLM {}

impl AsyncGenerateData for OpenAILLM {}
