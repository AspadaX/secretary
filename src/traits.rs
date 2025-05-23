use anyhow::{Error, Result, anyhow};
use async_openai::{
    Client,
    config::Config,
    types::{
        ChatCompletionRequestMessageContentPartTextArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequest, CreateChatCompletionRequestArgs, CreateChatCompletionResponse,
        ResponseFormat,
    },
};
use tokio::runtime::Runtime;

use crate::message_list::{Message, MessageList, Role};

/// Implement this for various LLM API standards
pub trait IsLLM {
    /// Provides access to the client instance.
    fn access_client(&self) -> &Client<impl Config>;

    /// Provides access to the model identifier.
    fn access_model(&self) -> &str;
}

/// Represent an object that has a system prompt
pub trait SystemPrompt {
    /// Get the system prompt
    fn get_system_prompt(&self) -> String;
}

/// Implement this for context management
pub trait Context {
    /// Update the context
    fn push(&mut self, role: Role, content: &str) -> Result<(), Error> {
        match role {
            Role::User => self
                .get_context_mut()
                .push(Message::new(Role::User, content.to_string())),
            Role::Assistant => {
                self.get_context_mut()
                    .push(Message::new(Role::Assistant, content.to_string()));
            }
            Role::System => {
                self.get_context_mut()
                    .push(Message::new(Role::System, content.to_string()));
            }
            _ => return Err(anyhow!("Unsupported role")),
        }

        Ok(())
    }

    /// Get access right to read and write the context
    fn get_context_mut(&mut self) -> &mut MessageList;

    /// Get a copy of the context
    fn get_context(&self) -> MessageList;
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
    fn generate_json(&self, task: &impl SystemPrompt, target: &str) -> Result<String, Error> {
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
                                    task.get_system_prompt()
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
    fn generate_json_with_context<T>(&self, task: &T) -> Result<String, Error>
    where
        T: SystemPrompt + Context,
    {
        let runtime: Runtime = tokio::runtime::Runtime::new()?;
        let result: String = runtime.block_on(async {
            let request: CreateChatCompletionRequest = CreateChatCompletionRequestArgs::default()
                .model(&self.access_model().to_string())
                .response_format(ResponseFormat::JsonObject)
                .messages(task.get_context())
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
    /// Asynchronously generates JSON response from the LLM based on the provided prompt.
    ///
    /// This is the asynchronous version of `generate_json` that can be used in async contexts.
    ///
    /// # Arguments
    ///
    /// * `task` - An implementation of `SystemPrompt` containing schema and instructions.
    /// * `target` - A string slice that holds the data to be sent to the LLM to generate a json.
    ///
    /// # Returns
    ///
    /// * `Result<String, Error>` - A result containing the JSON response as a string or an error.
    ///
    /// # Example
    ///
    /// ```
    /// use secretary::{openai::OpenAILLM, tasks::basic_task::BasicTask, traits::AsyncGenerateJSON};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Debug, Serialize, Deserialize)]
    /// struct MyData {
    ///     field: String,
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let llm = OpenAILLM::new("api_base", "api_key", "model")?;
    ///     let task = BasicTask::new(
    ///         MyData { field: "Description for field".to_string() },
    ///         vec!["Extract data from the text".to_string()],
    ///     );
    ///     
    ///     let result = llm.async_generate_json(&task, "Some text with info").await?;
    ///     println!("{}", result);
    ///     Ok(())
    /// }
    /// ```
    async fn async_generate_json(
        &self,
        task: &impl SystemPrompt,
        target: &str,
    ) -> Result<String, Error> {
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.access_model().to_string())
            .response_format(ResponseFormat::JsonObject)
            .messages(vec![
                ChatCompletionRequestUserMessageArgs::default()
                    .content(vec![
                        ChatCompletionRequestMessageContentPartTextArgs::default()
                            .text(
                                task.get_system_prompt()
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

    /// Asynchronously generates JSON response from the LLM based on the provided context.
    ///
    /// This is the asynchronous version of `generate_json_with_context` that enables
    /// context-aware conversations in async contexts.
    ///
    /// # Arguments
    ///
    /// * `task` - An implementation of both `SystemPrompt` and `Context` traits that provides
    ///            both the schema definition and conversation history.
    ///
    /// # Returns
    ///
    /// * `Result<String, Error>` - A result containing the JSON response as a string or an error.
    ///
    /// # Example
    ///
    /// ```
    /// use secretary::{
    ///     message_list::Role,
    ///     openai::OpenAILLM,
    ///     tasks::basic_task::BasicTask,
    ///     traits::{AsyncGenerateJSON, Context},
    /// };
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Debug, Serialize, Deserialize)]
    /// struct MyData {
    ///     field: String,
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let llm = OpenAILLM::new("api_base", "api_key", "model")?;
    ///     let mut task = BasicTask::new(
    ///         MyData { field: "Description for field".to_string() },
    ///         vec!["Extract data from the text".to_string()],
    ///     );
    ///     
    ///     // Add messages to the conversation context
    ///     task.push(Role::User, "Here's my first message")?;
    ///     
    ///     // Generate response with context
    ///     let result = llm.async_generate_json_with_context(&task).await?;
    ///     println!("{}", result);
    ///     Ok(())
    /// }
    /// ```
    async fn async_generate_json_with_context<T>(&self, task: &T) -> Result<String, Error>
    where
        T: SystemPrompt + Context,
    {
        let request: CreateChatCompletionRequest = CreateChatCompletionRequestArgs::default()
            .model(&self.access_model().to_string())
            .response_format(ResponseFormat::JsonObject)
            .messages(task.get_context())
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
