use serde_json::{Value, json};

use crate::{
    constants::{
        AZURE_OPENAI_API_VERSION_MARKER, AZURE_OPENAI_COMPLETION_ROUTE,
        AZURE_OPENAI_DEPLOYMENT_ID_MARKER,
    },
    message::Message,
    traits::{AsyncGenerateData, GenerateData, IsLLM},
};

/// Represents a Large Language Model (LLM) that is compatible with OpenAI API.
/// An LLM is the primary tool we use to convert unstructured data into structured data.
#[derive(Debug, Clone)]
pub struct AzureOpenAILLM {
    model: String,
    base_url: String,
    api_key: String,
}

impl AzureOpenAILLM {
    /// Creates a new instance of the AzureOpenAILLM struct.
    ///
    /// # Arguments
    ///
    /// * `api_base` - A string slice that holds the base URL for the Azure OpenAI API.
    /// * `api_key` - A string slice that holds the API key for authenticating with the Azure OpenAI API.
    /// * `deployment_id` - A string slice that specifies the deployment ID for the Azure OpenAI service.
    /// * `api_version` - A string slice that specifies the API version to use.
    ///
    /// # Returns
    ///
    /// * `Result<Self, Error>` - On success, returns an instance of the AzureOpenAILLM struct. On failure, returns an Box<dyn std::error::Error>.
    pub fn new(api_base: &str, api_key: &str, deployment_id: &str, api_version: &str) -> Self {
        let base_url: String = AZURE_OPENAI_COMPLETION_ROUTE
            .replace(AZURE_OPENAI_COMPLETION_ROUTE, api_base)
            .replace(AZURE_OPENAI_API_VERSION_MARKER, api_version)
            .replace(AZURE_OPENAI_DEPLOYMENT_ID_MARKER, deployment_id);

        Self {
            model: deployment_id.to_string(),
            base_url,
            api_key: api_key.to_string(),
        }
    }
}

impl IsLLM for AzureOpenAILLM {
    fn get_authorization_credentials(&self) -> String {
        self.api_key.clone()
    }

    fn get_model_ref(&self) -> &str {
        &self.model
    }

    fn get_chat_completion_request_url(&self) -> String {
        self.base_url.clone()
    }

    fn get_request_body(&self, message: Message, return_json: bool) -> Value {
        if return_json {
            return json!(
                {
                    "messages": [message],
                    "response_format": {"type": "json_object"}
                }
            );
        }

        return json!(
            {
                "messages": [message],
            }
        );
    }
}

impl GenerateData for AzureOpenAILLM {}

impl AsyncGenerateData for AzureOpenAILLM {}
