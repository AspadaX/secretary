use anyhow::{Error, Result, anyhow};
use async_openai::{
    Client,
    config::Config,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestMessageContentPartTextArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
        CreateChatCompletionResponse, ResponseFormat,
    },
};

use crate::{
    message_list::{Message, MessageList, Role},
    prompt::Prompt,
};

/// Implement this for various LLM API standards
pub trait IsLLM {
    /// Provides access to the client instance.
    fn access_client(&self) -> &Client<impl Config>;

    /// Provides access to the model identifier.
    fn access_model(&self) -> &str;
}

/// Implement this for context management
pub trait Context {
    /// Update the context
    fn push(&mut self, role: async_openai::types::Role, content: &str) -> Result<(), Error> {
        match role {
            async_openai::types::Role::User => self
                .get_context_mut()
                .push(Message::new(Role::User, content.to_string())),
            async_openai::types::Role::Assistant => {
                self.get_context_mut()
                    .push(Message::new(Role::Assistant, content.to_string()));
            }
            async_openai::types::Role::System => {
                self.get_context_mut()
                    .push(Message::new(Role::System, content.to_string()));
            }
            _ => return Err(anyhow!("Unsupported role")),
        }

        Ok(())
    }

    /// Get access right to read and write the context
    fn get_context_mut(&mut self) -> &mut MessageList;
}

pub trait GenerateJSON
where
    Self: IsLLM,
{
    /// Generates JSON response from the LLM based on the provided prompt.
    ///
    /// # Arguments
    ///
    /// * `prompt` - A string slice that holds the prompt to be sent to the LLM.
    /// * `target` - A string slice that holds the data to be sent to the LLM to generate a json.
    ///
    /// # Returns
    ///
    /// * `Result<String, Error>` - A result containing the JSON response as a string or an error.
    fn generate_json(&self, prompt: &Prompt, target: &str) -> Result<String, Error> {
        let runtime = tokio::runtime::Runtime::new()?;
        let result: String = runtime.block_on(async {
            let request = CreateChatCompletionRequestArgs::default()
                .model(&self.access_model().to_string())
                .response_format(ResponseFormat::JsonObject)
                .messages(vec![
                    ChatCompletionRequestUserMessageArgs::default()
                        .content(vec![
                            ChatCompletionRequestMessageContentPartTextArgs::default()
                                .text(
                                    prompt.to_string()
                                        + "\nThis is the basis for generating a json:\n"
                                        + target,
                                )
                                .build()?
                                .into(),
                        ])
                        .build()?
                        .into(),
                ])
                .build()?;

            let response: CreateChatCompletionResponse =
                match self.access_client().chat().create(request.clone()).await {
                    std::result::Result::Ok(response) => response,
                    Err(e) => {
                        anyhow::bail!("Failed to execute function: {}", e);
                    }
                };

            if let Some(content) = response.choices[0].clone().message.content {
                return Ok(content);
            }

            return Err(anyhow!("No response is retrieved from the LLM"));
        })?;

        Ok(result)
    }

    /// Generates JSON response from the LLM based on the provided context.
    ///
    /// # Arguments
    ///
    /// * `context` - A collection of `ChatCompletionRequestMessage` instances that provide the context to be sent to the LLM.
    ///
    /// # Returns
    ///
    /// * `Result<String, Error>` - A result containing the JSON response as a string or an error.
    fn generate_json_with_context(
        &self,
        context: impl Into<Vec<ChatCompletionRequestMessage>>,
    ) -> Result<String, Error> {
        let runtime = tokio::runtime::Runtime::new()?;
        let result = runtime.block_on(async {
            let request = CreateChatCompletionRequestArgs::default()
                .model(&self.access_model().to_string())
                .response_format(ResponseFormat::JsonObject)
                .messages(context)
                .build()?;

            let response: CreateChatCompletionResponse =
                match self.access_client().chat().create(request.clone()).await {
                    std::result::Result::Ok(response) => response,
                    Err(e) => {
                        anyhow::bail!("Failed to execute function: {}", e);
                    }
                };

            if let Some(content) = response.choices[0].clone().message.content {
                return Ok(content);
            }

            return Err(anyhow!("No response is retrieved from the LLM"));
        })?;

        Ok(result)
    }
}

pub trait AsyncGenerateJSON
where
    Self: IsLLM,
{
    // TODO: Docs and Examples
    async fn async_generate_json(&self, prompt: &Prompt, target: &str) -> Result<String, Error> {
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.access_model().to_string())
            .response_format(ResponseFormat::JsonObject)
            .messages(vec![
                ChatCompletionRequestUserMessageArgs::default()
                    .content(vec![
                        ChatCompletionRequestMessageContentPartTextArgs::default()
                            .text(
                                prompt.to_string()
                                    + "\nThis is the basis for generating a json:\n"
                                    + target,
                            )
                            .build()?
                            .into(),
                    ])
                    .build()?
                    .into(),
            ])
            .build()?;

        let response: CreateChatCompletionResponse =
            match self.access_client().chat().create(request.clone()).await {
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

    // TODO: Docs and Examples
    async fn async_generate_json_with_context(
        &self,
        context: impl Into<Vec<ChatCompletionRequestMessage>>,
    ) -> Result<String, Error> {
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.access_model().to_string())
            .response_format(ResponseFormat::JsonObject)
            .messages(context)
            .build()?;

        let response: CreateChatCompletionResponse =
            match self.access_client().chat().create(request.clone()).await {
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
}
