use async_trait::async_trait;
use reqwest::{header::{AUTHORIZATION, CONTENT_TYPE}, Response};
use serde::{Deserialize, Serialize};

use serde_json::Value;

// Re-export the derive macro
pub use secretary_derive::Task;

use crate::{SecretaryError, message::Message};

/// Implement this for various LLM API standards
#[async_trait]
pub trait IsLLM {
    fn send_message(
        &self,
        message: Message,
        return_json: bool,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let authorization_credentials: (String, String) = self.get_authorization_credentials();
        let request: reqwest::blocking::Response = reqwest::blocking::Client::new()
            .post(self.get_chat_completion_request_url())
            .header(AUTHORIZATION, authorization_credentials.1)
            .header(CONTENT_TYPE, "application/json")
            .json(&self.get_reqeust_body(message, return_json))
            .send()?;
        
        Ok(request.text()?)
    }
    
    /// Get raw response from an LLM
    async fn async_send_message(
        &self,
        message: Message,
        return_json: bool,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let authorization_credentials: (String, String) = self.get_authorization_credentials();
        let request: Response = reqwest::Client::new()
            .post(self.get_chat_completion_request_url())
            .header(AUTHORIZATION, authorization_credentials.1)
            .header(CONTENT_TYPE, "application/json")
            .json(&self.get_reqeust_body(message, return_json))
            .send()
            .await?;
        
        Ok(request.text().await?)
    }

    fn get_authorization_credentials(&self) -> (String, String);

    fn get_reqeust_body(&self, message: Message, return_json: bool) -> Value;

    /// Provide a chat completion url
    fn get_chat_completion_request_url(&self) -> String;

    /// Provides reference to the model identifier.
    fn get_model_ref(&self) -> &str;
}

/// The main Task trait that combines data model, system prompt, and context functionality.
/// This trait should be implemented using the derive macro for user-defined structs.
pub trait Task: Serialize + for<'de> Deserialize<'de> + Default {
    /// Get the system prompt (combines SystemPrompt functionality)
    fn get_system_prompt(&self) -> String;
}

pub trait GenerateData
where
    Self: IsLLM + Sync,
{
    /// Generates JSON response from the LLM based on the provided prompt.
    ///
    /// # Arguments
    ///
    /// * `task` - A Task implementation that provides the system prompt and schema.
    /// * `target` - A string slice that holds the data to be sent to the LLM to generate a json.
    ///
    /// # Returns
    ///
    /// * `Result<String, Box<dyn std::error::Error + Send + Sync + 'static>>` - A result containing the JSON response as a string or an Box<dyn std::error::Error + Send + Sync + 'static>.
    fn generate_data<T: Task>(
        &self,
        task: &T,
        target: &str,
        additional_instructions: &Vec<String>,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let formatted_additional_instructions: String =
            format_additional_instructions(additional_instructions);
        let request: String = self.send_message(
            Message {
                role: "user".to_string(),
                content: format!(
                    "{}{}\nThis is the basis for generating a json:\n{}",
                    task.get_system_prompt(),
                    formatted_additional_instructions,
                    target
                ),
            },
            true,
        )?;

        let value: Value = serde_json::from_str(&request).unwrap();
        let result = value["choices"][0]["message"]["content"]
            .as_str()
            .unwrap()
            .to_string();

        Ok(serde_json::from_str::<T>(&result)?)
    }

    fn force_generate_data<T: Task>(
        &self,
        task: &T,
        target: &str,
        additional_instructions: &Vec<String>,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let formatted_additional_instructions: String =
            format_additional_instructions(additional_instructions);

        let response: String = self
            .send_message(
                Message {
                    role: "user".to_string(),
                    content: format!(
                        "{}{}\nThis is the basis for generating a json:\n{}",
                        task.get_system_prompt(),
                        formatted_additional_instructions,
                        target
                    ),
                },
                false,
            )?;

        let value: Value = serde_json::from_str(&response).unwrap();
        let result: String = value["choices"][0]["message"]["content"]
            .as_str()
            .unwrap()
            .to_string();

        Ok(surfing::serde::from_mixed_text(&result)?)
    }
}

#[async_trait]
pub trait AsyncGenerateData
where
    Self: IsLLM,
{
    /// Asynchronously generates JSON response from the LLM based on the provided prompt.
    ///
    /// This is the asynchronous version of `generate_json` that can be used in async contexts.
    ///
    /// # Arguments
    ///
    /// * `task` - A Task implementation that provides the system prompt and schema.
    /// * `target` - A string slice that holds the data to be sent to the LLM to generate a json.
    ///
    /// ::<String, Box<dyn std::error::Error + Send + Sync + 'static>># Returns
    ///
    /// * `Result<String, Box<dyn std::error::Error + Send + Sync + 'static>>` - A result containing the JSON response as a string or an Box<dyn std::error::Error + Send + Sync + 'static>.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use secretary::llm_providers::openai::OpenAILLM;
    /// use secretary::traits::{AsyncGenerateData, Task};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Task, Debug, Serialize, Deserialize, Default)]
    /// struct MyData {
    ///     #[task(instruction = "Extract the field value")]
    ///     field: String,
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let llm = OpenAILLM::new("api_base", "api_key", "model")?;
    ///     let task = MyData::new(vec![]);
    ///     
    ///     let result: MyData = llm.async_generate_data(&task, "Some text with info").await?;
    ///     println!("{:#?}", result);
    ///     Ok(())
    /// }
    /// ```
    async fn async_generate_data<T: Task + Sync + Send>(
        &self,
        task: &T,
        target: &str,
        additional_instructions: &Vec<String>,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let formatted_additional_instructions: String =
            format_additional_instructions(additional_instructions);

        let request: Result<String, Box<dyn std::error::Error + Send + Sync>> = self
            .async_send_message(
                Message {
                    role: "user".to_string(),
                    content: format!(
                        "{}{}\nThis is the basis for generating a json:\n{}",
                        task.get_system_prompt(),
                        formatted_additional_instructions,
                        target
                    ),
                },
                true,
            )
            .await;

        let result = match request {
            Ok(result) => {
                dbg!(&result);
                let value: Value = serde_json::from_str(&result).unwrap();
                value["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap()
                    .to_string()
            }
            Err(error) => return Err(SecretaryError::BuildRequestError(error.to_string()).into()),
        };

        Ok(serde_json::from_str(&result)?)
    }

    async fn async_force_generate_data<T: Task + Send + Sync>(
        &self,
        task: &T,
        target: &str,
        additional_instructions: &Vec<String>,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let formatted_additional_instructions: String =
            format_additional_instructions(additional_instructions);

        let request: Result<String, Box<dyn std::error::Error + Send + Sync>> = self
            .async_send_message(
                Message {
                    role: "user".to_string(),
                    content: format!(
                        "{}{}\nThis is the basis for generating a json:\n{}",
                        task.get_system_prompt(),
                        formatted_additional_instructions,
                        target
                    ),
                },
                false,
            )
            .await;

        let result: String = match request {
            Ok(result) => {
                let value: Value = serde_json::from_str(&result).unwrap();
                value["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap()
                    .to_string()
            }
            Err(error) => return Err(SecretaryError::BuildRequestError(error.to_string()).into()),
        };

        Ok(surfing::serde::from_mixed_text(&result)?)
    }
}

fn format_additional_instructions(additional_instructions: &Vec<String>) -> String {
    let mut prompt: String = String::new();
    // Add additional instructions
    if !additional_instructions.is_empty() {
        prompt.push_str("\nAdditional instructions:\n");
        for instruction in additional_instructions {
            prompt.push_str(&format!("- {}\n", instruction));
        }
    }

    prompt
}
