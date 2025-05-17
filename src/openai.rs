use anyhow::{Error, Result};
use async_openai::Client;
use async_openai::config::OpenAIConfig;

use crate::traits::{AsyncGenerateJSON, GenerateJSON, IsLLM};

/// Represents a Large Language Model (LLM) that is compatible with OpenAI API.
/// An LLM is the primary tool we use to convert unstructured data into structured data.
#[derive(Debug)]
pub struct OpenAILLM {
    model: String,
    client: Client<OpenAIConfig>,
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
    /// * `Result<Self, Error>` - On success, returns an instance of the LLM struct. On failure, returns an Error.
    pub fn new(api_base: &str, api_key: &str, model: &str) -> Result<Self, Error> {
        let llm_configuration: OpenAIConfig = OpenAIConfig::default()
            .with_api_key(api_key)
            .with_api_base(api_base);
        let client: Client<OpenAIConfig> = async_openai::Client::with_config(llm_configuration);

        Ok(Self {
            model: model.to_string(),
            client,
        })
    }
}

impl IsLLM for OpenAILLM {
    fn access_client(&self) -> &Client<impl async_openai::config::Config> {
        &self.client
    }

    fn access_model(&self) -> &str {
        &self.model
    }
}

impl GenerateJSON for OpenAILLM {}

impl AsyncGenerateJSON for OpenAILLM {}
