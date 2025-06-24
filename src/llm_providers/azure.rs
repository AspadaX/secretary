use anyhow::{Error, Result};
use async_openai::Client;
use async_openai::config::AzureConfig;

use crate::traits::{AsyncGenerateData, GenerateData, IsLLM};

/// Represents a Large Language Model (LLM) that is compatible with OpenAI API.
/// An LLM is the primary tool we use to convert unstructured data into structured data.
#[derive(Debug, Clone)]
pub struct AzureOpenAILLM {
    model: String,
    client: Client<AzureConfig>,
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
    /// * `Result<Self, Error>` - On success, returns an instance of the AzureOpenAILLM struct. On failure, returns an Error.
    pub fn new(api_base: &str, api_key: &str, deployment_id: &str, api_version: &str) -> Result<Self, Error> {
        let llm_configuration: AzureConfig = AzureConfig::default()
            .with_deployment_id(deployment_id)
            .with_api_version(api_version)
            .with_api_key(api_key)
            .with_api_base(api_base);
        let client: Client<AzureConfig> = async_openai::Client::with_config(llm_configuration);

        Ok(Self {
            model: deployment_id.to_string(),
            client,
        })
    }
}

impl IsLLM for AzureOpenAILLM {
    fn access_client(&self) -> &Client<impl async_openai::config::Config> {
        &self.client
    }

    fn access_model(&self) -> &str {
        &self.model
    }
}

impl GenerateData for AzureOpenAILLM {}

impl AsyncGenerateData for AzureOpenAILLM {}