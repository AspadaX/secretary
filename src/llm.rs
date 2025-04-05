
use anyhow::anyhow;
use anyhow::{Error, Result};
use async_openai::{config::OpenAIConfig, types::{ChatCompletionRequestMessageContentPartTextArgs, ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs, CreateChatCompletionResponse}};
use async_openai::Client;

use crate::traits::{GenerateJSON, IsLLM};

/// Represents a Large Language Model (LLM).
/// An LLM is the primary tool we use to convert unstructured data into structured data.
#[derive(Debug)]
pub struct LLM {
    model: String,
    client: Client<OpenAIConfig>
}

impl LLM {
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
        let client: Client<OpenAIConfig> = async_openai::Client::with_config(
            llm_configuration
        );

        Ok(Self { model: model.to_string(), client})
    }

    pub fn generate(&self, prompt: String) -> Result<String, Error> {
        let runtime = tokio::runtime::Runtime::new()?;
        let result = runtime.block_on(
            async {
                let request = CreateChatCompletionRequestArgs::default()
                    .model(&self.model)
                    .messages(vec![ChatCompletionRequestUserMessageArgs::default()
                        .content(vec![
                            ChatCompletionRequestMessageContentPartTextArgs::default()
                                .text(prompt)
                                .build()?
                                .into(),
                        ])
                        .build()?
                        .into()])
                    .build()?;

                let response: CreateChatCompletionResponse =
                    match self.client.chat().create(request.clone()).await {
                        std::result::Result::Ok(response) => response,
                        Err(e) => {
                            anyhow::bail!("Failed to execute function: {}", e);
                        }
                    };
                
                if let Some(content) = response.choices[0].clone().message.content {
                    return Ok(content);
                }

                return Err(anyhow!("No response is retrieved from the LLM"));
            }
        )?;

        Ok(result)
    }
}

impl IsLLM for LLM {
    fn access_client(&self) -> &Client<impl async_openai::config::Config> {
        &self.client
    }
    
    fn access_model(&self) -> &str {
        &self.model
    }
}

impl GenerateJSON for LLM {}