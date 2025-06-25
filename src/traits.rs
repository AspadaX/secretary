use serde::{Deserialize, Serialize};
use serde_json::Value;

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

// Re-export the derive macro
pub use secretary_derive::Task;

/// Implement this for various LLM API standards
pub trait IsLLM {
    /// Provides access to the client instance.
    fn access_client(&self) -> &Client<impl Config>;

    /// Provides access to the model identifier.
    fn access_model(&self) -> &str;
}

/// The main Task trait that combines data model, system prompt, and context functionality.
/// This trait should be implemented using the derive macro for user-defined structs.
pub trait Task: Serialize + for<'de> Deserialize<'de> + Default {
    /// Get the data model in JSON format with instructions specified.
    fn get_data_model_instructions() -> Value {
        serde_json::to_value(Self::provide_data_model_instructions())
            .expect("Failed to convert data model to JSON")
    }

    /// Get the data model with instructions specified, which will be used
    /// to instruct the LLM for what to generate. Typically, this is the only method
    /// you need to implement in the DataModel trait.
    ///
    /// ```rust
    /// use secretary::traits::Task;
    /// use secretary::message_list::MessageList;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Serialize, Deserialize, Default)]
    /// pub struct Example {
    ///     field: String,
    /// }
    ///
    /// impl Task for Example {
    ///    fn provide_data_model_instructions() -> Self {
    ///        Example {
    ///           field: "Extract the field of the subject and put it here".to_string(),
    ///        }
    ///    }
    ///    
    ///    fn get_system_prompt(&self) -> String {
    ///        "Extract data".to_string()
    ///    }
    /// }
    ///
    /// ```
    fn provide_data_model_instructions() -> Self;

    /// Get the system prompt (combines SystemPrompt functionality)
    fn get_system_prompt(&self) -> String;
}

pub trait GenerateData
where
    Self: IsLLM,
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
    /// * `Result<String, Error>` - A result containing the JSON response as a string or an error.
    fn generate_data<T: Task>(&self, task: &T, target: &str, additional_instructions: &Vec<String>) -> Result<T, Error> {
        let formatted_additional_instructions: String = format_additional_instructions(additional_instructions);
        let runtime: Runtime = tokio::runtime::Runtime::new()?;
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
                                        + &formatted_additional_instructions
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

        Ok(serde_json::from_str(&result)?)
    }
}

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
    /// # Returns
    ///
    /// * `Result<String, Error>` - A result containing the JSON response as a string or an error.
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
    async fn async_generate_data<T: Task>(&self, task: &T, target: &str, additional_instructions: &Vec<String>) -> Result<T, Error> {
        let formatted_additional_instructions: String = format_additional_instructions(additional_instructions);
        let request: CreateChatCompletionRequest = CreateChatCompletionRequestArgs::default()
            .model(&self.access_model().to_string())
            .response_format(ResponseFormat::JsonObject)
            .messages(vec![
                ChatCompletionRequestUserMessageArgs::default()
                    .content(vec![
                        ChatCompletionRequestMessageContentPartTextArgs::default()
                            .text(
                                task.get_system_prompt()
                                    + &formatted_additional_instructions
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
            return Ok(serde_json::from_str(&content)?);
        }

        return Err(anyhow!("No response is retrieved from the LLM"));
    }
}

/// Enables serialization of data structures to JSON format.
///
/// Useful for persistence and state transfer between service instances.
/// Particularly valuable in web services where object lifetime management
/// is important and state needs to be reconstructed from client responses.
///
/// This trait provides a simple interface for converting any type that implements
/// `serde::Serialize` into a JSON string representation. It's designed to work
/// alongside the complementary `FromJSON` trait for full serialization/deserialization
/// capabilities.
///
/// # Examples
///
/// ```
/// use secretary::traits::ToJSON;
///
/// #[derive(serde::Serialize)]
/// struct User {
///     name: String,
///     email: String,
/// }
///
/// impl ToJSON for User {}
///
/// fn main() -> anyhow::Result<()> {
///     let user = User {
///         name: "Alice".to_string(),
///         email: "alice@example.com".to_string(),
///     };
///     
///     let json = user.to_json()?;
///     println!("{}", json); // Outputs: {"name":"Alice","email":"alice@example.com"}
///     Ok(())
/// }
/// ```
pub trait ToJSON
where
    Self: serde::Serialize + Sized,
{
    fn to_json(&self) -> Result<String, Error> {
        Ok(serde_json::to_string(self)?)
    }
}

/// Enables creation of types from JSON string representation.
///
/// This trait provides deserialization capabilities complementary to
/// the `ToJSON` trait. It's particularly useful for reconstructing
/// objects from client-provided JSON data in web services or for
/// loading persisted application state.
///
/// # Examples
///
/// ```
/// use secretary::traits::FromJSON;
///
/// #[derive(serde::Deserialize)]
/// struct User {
///     name: String,
///     email: String,
/// }
///
/// impl FromJSON for User {}
///
/// fn main() -> anyhow::Result<()> {
///     let json = r#"{"name":"Alice","email":"alice@example.com"}"#;
///     let user = User::from_json(json)?;
///     assert_eq!(user.name, "Alice");
///     assert_eq!(user.email, "alice@example.com");
///     Ok(())
/// }
/// ```
pub trait FromJSON {
    fn from_json(json: &str) -> Result<Self, Error>
    where
        Self: for<'de> serde::Deserialize<'de> + Sized,
    {
        Ok(serde_json::from_str(json)?)
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
