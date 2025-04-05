use anyhow::{anyhow, Error, Result};
use async_openai::{config::Config, types::{ChatCompletionRequestMessage, ChatCompletionRequestMessageContentPartTextArgs, ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs, CreateChatCompletionResponse, ResponseFormat}, Client};

pub trait IsLLM {
    /// Provides access to the client instance.
    fn access_client(&self) -> &Client<impl Config>;

    /// Provides access to the model identifier.
    fn access_model(&self) -> &str;
}

pub trait GenerateJSON
where 
    Self: IsLLM
{
    /// Generates JSON response from the LLM based on the provided prompt.
    ///
    /// # Arguments
    ///
    /// * `prompt` - A string slice that holds the prompt to be sent to the LLM.
    ///
    /// # Returns
    ///
    /// * `Result<String, Error>` - A result containing the JSON response as a string or an error.
    fn generate_json(&self, prompt: String) -> Result<String, Error> {
        let runtime = tokio::runtime::Runtime::new()?;
        let result = runtime.block_on(
            async {
                let request = CreateChatCompletionRequestArgs::default()
                    .model(&self.access_model().to_string())
                    .response_format(ResponseFormat::JsonObject)
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
        )?;

        Ok(result)
    }

    /// Generates JSON response from the LLM based on the provided context.
    ///
    /// # Arguments
    ///
    /// * `context` - A vector of `ChatCompletionRequestMessage` that holds the context to be sent to the LLM.
    ///
    /// # Returns
    ///
    /// * `Result<String, Error>` - A result containing the JSON response as a string or an error.
    fn generate_json_with_context(&self, context: Vec<ChatCompletionRequestMessage>) -> Result<String, Error> {
        let runtime = tokio::runtime::Runtime::new()?;
        let result = runtime.block_on(
            async {
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
        )?;

        Ok(result)
    }
}