use std::panic;

use async_trait::async_trait;
use reqwest::{
    Response,
    header::{AUTHORIZATION, CONTENT_TYPE},
};
use serde::{Deserialize, Serialize};

use serde_json::Value;

// Re-export the derive macro
pub use secretary_derive::Task;
use tokio::io::join;

use crate::{generate_from_tuples, message::Message, SecretaryError};

/// Core trait for implementing LLM providers that are compatible with OpenAI-style APIs.
///
/// This trait provides the foundation for integrating different LLM services by defining
/// the essential methods needed for authentication, request formatting, and communication.
///
/// # Examples
///
/// ```rust
/// # use secretary::llm_providers::openai::OpenAILLM;
/// # use secretary::llm_providers::azure::AzureOpenAILLM;
/// # fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
/// // OpenAI provider
/// let openai_llm = OpenAILLM::new(
///     "https://api.openai.com/v1",
///     "your-api-key",
///     "gpt-4"
/// )?;
///
/// // Azure OpenAI provider
/// let azure_llm = AzureOpenAILLM::new(
///     "https://your-resource.openai.azure.com",
///     "your-api-key",
///     "your-deployment-id",
///     "2024-02-15-preview"
/// );
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait IsLLM {
    /// Sends a synchronous message to the LLM and returns the raw response.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send to the LLM
    /// * `return_json` - Whether to request JSON format response (enables JSON mode if supported)
    ///
    /// # Returns
    ///
    /// Raw response string from the LLM API
    fn send_message(
        &self,
        message: Message,
        return_json: bool,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let request: reqwest::blocking::Response = reqwest::blocking::Client::new()
            .post(self.get_chat_completion_request_url())
            .header(AUTHORIZATION, self.get_authorization_credentials())
            .header(CONTENT_TYPE, "application/json")
            .json(&self.get_reqeust_body(message, return_json))
            .send()?;

        Ok(request.text()?)
    }

    /// Sends an asynchronous message to the LLM and returns the raw response.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send to the LLM
    /// * `return_json` - Whether to request JSON format response (enables JSON mode if supported)
    ///
    /// # Returns
    ///
    /// Raw response string from the LLM API
    async fn async_send_message(
        &self,
        message: Message,
        return_json: bool,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let request: Response = reqwest::Client::new()
            .post(self.get_chat_completion_request_url())
            .header(AUTHORIZATION, self.get_authorization_credentials())
            .header(CONTENT_TYPE, "application/json")
            .json(&self.get_reqeust_body(message, return_json))
            .send()
            .await?;

        Ok(request.text().await?)
    }

    /// Returns the authorization credentials for the LLM provider.
    ///
    /// # Returns
    ///
    /// A tuple of (header_name, header_value) for authentication
    fn get_authorization_credentials(&self) -> String;

    /// Constructs the request body for the LLM API call.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to include in the request
    /// * `return_json` - Whether to enable JSON mode in the request
    ///
    /// # Returns
    ///
    /// JSON value representing the request body
    fn get_reqeust_body(&self, message: Message, return_json: bool) -> Value;

    /// Returns the complete URL for the chat completion endpoint.
    ///
    /// # Returns
    ///
    /// The full URL string for making API requests
    fn get_chat_completion_request_url(&self) -> String;

    /// Returns a reference to the model identifier being used.
    ///
    /// # Returns
    ///
    /// String slice containing the model name or deployment ID
    fn get_model_ref(&self) -> &str;
}

/// The main Task trait for defining data extraction schemas and system prompts.
///
/// This trait should be implemented using the `#[derive(Task)]` macro for user-defined structs.
/// It combines data model definition, system prompt generation, and serialization capabilities
/// into a single, cohesive interface for LLM-based data extraction.
///
/// # Examples
///
/// ```rust
/// use secretary::Task;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Task, Serialize, Deserialize, Debug)]
/// struct PersonInfo {
///     #[task(instruction = "Extract the person's full name")]
///     pub name: String,
///     
///     #[task(instruction = "Extract age as a number")]
///     pub age: u32,
///     
///     #[task(instruction = "Extract email address if mentioned")]
///     pub email: Option<String>,
/// }
///
/// let task = PersonInfo::new();
/// let system_prompt = task.get_system_prompt();
/// println!("Generated prompt: {}", system_prompt);
/// ```
///
/// # Derive Macro
///
/// The `#[derive(Task)]` macro automatically implements this trait and generates:
/// - System prompts based on field instructions
/// - JSON schema definitions
/// - A `new()` constructor method
///
/// Use `#[task(instruction = "...")]` attributes on fields to provide extraction guidance.
pub trait Task: Serialize + for<'de> Deserialize<'de> + Default {
    /// Generates a system prompt for the LLM based on the struct's field instructions.
    ///
    /// This method creates a comprehensive prompt that includes:
    /// - JSON structure specifications
    /// - Field-specific extraction instructions
    /// - Response format requirements
    ///
    /// # Returns
    ///
    /// A formatted string containing the complete system prompt
    fn get_system_prompt(&self) -> String;
    
    /// # Returns:
    /// a field name and a prompt
    fn get_system_prompts_for_distributed_generation(&self) -> Vec<(String, String)>;
    
    /// Create a prompt that will be sending to the LLM for generating a structural data
    fn make_prompt(&self, target: &str, additional_instructions: &Vec<String>) -> Message {
        Message {
            role: "user".to_string(),
            content: format!(
                "{}{}\nThis is the basis for generating a json:\n{}",
                self.get_system_prompt(),
                format_additional_instructions(additional_instructions),
                target
            ),
        }
    }
    
    /// Create a prompt that will be sending to the LLM for generating a structural data
    fn make_dstributed_generation_prompts(&self, target: &str, additional_instructions: &Vec<String>) -> Vec<(String, Message)> {
        let mut messages: Vec<(String, Message)> = Vec::new();
        
        for prompt in self.get_system_prompts_for_distributed_generation() {
            messages.push(
                (
                    prompt.0,
                    Message {
                        role: "user".to_string(),
                        content: format!(
                            "{}{}\nThis is the basis for generating the result:\n{}",
                            prompt.1,
                            format_additional_instructions(additional_instructions),
                            target
                        ),
                    }
                )
            );
        }
        
        messages
    }
}

/// Trait for synchronous data generation from LLMs.
///
/// This trait provides methods for extracting structured data from natural language text
/// using LLM providers. It includes both standard JSON mode generation and force generation
/// for reasoning models that don't support JSON mode.
///
/// # Examples
///
/// ```no_run
/// use secretary::Task;
/// use secretary::llm_providers::openai::OpenAILLM;
/// use secretary::traits::GenerateData;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Task, Serialize, Deserialize, Debug)]
/// struct ProductInfo {
///     #[task(instruction = "Extract the product name")]
///     pub name: String,
///     #[task(instruction = "Extract price as a number")]
///     pub price: f64,
/// }
///
/// # fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
/// let llm = OpenAILLM::new(
///     "https://api.openai.com/v1",
///     "your-api-key",
///     "gpt-4"
/// )?;
///
/// let task = ProductInfo::new();
/// let additional_instructions = vec!["Be precise with pricing".to_string()];
///
/// let input = "Apple MacBook Pro 16-inch costs $2,499";
/// let result: ProductInfo = llm.generate_data(&task, input, &additional_instructions)?;
///
/// println!("Extracted: {:#?}", result);
/// # Ok(())
/// # }
/// ```
pub trait GenerateData
where
    Self: IsLLM + Sync,
{
    /// Generates structured data from natural language using JSON mode.
    ///
    /// This method uses the LLM's JSON mode (if available) to ensure structured output.
    /// For models without JSON mode support, use `force_generate_data` instead.
    ///
    /// # Arguments
    ///
    /// * `task` - A Task implementation that provides the system prompt and schema
    /// * `target` - The natural language text to extract data from
    /// * `additional_instructions` - Extra instructions to guide the extraction process
    ///
    /// # Returns
    ///
    /// A Result containing the extracted data as the specified type T
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The LLM API call fails
    /// - The response cannot be parsed as valid JSON
    /// - The JSON doesn't match the expected schema
    fn generate_data<T: Task>(
        &self,
        task: &T,
        target: &str,
        additional_instructions: &Vec<String>,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let request: String = self.send_message(
            task.make_prompt(target, additional_instructions),
            true,
        )?;

        let value: Value = serde_json::from_str(&request).unwrap();
        let result = value["choices"][0]["message"]["content"]
            .as_str()
            .unwrap()
            .to_string();

        Ok(serde_json::from_str::<T>(&result)?)
    }

    /// Generates structured data from natural language without JSON mode (for reasoning models).
    ///
    /// This method is designed for reasoning models like o1, deepseek, and others that don't
    /// support JSON mode. It uses text parsing to extract JSON from the model's response.
    ///
    /// # Arguments
    ///
    /// * `task` - A Task implementation that provides the system prompt and schema
    /// * `target` - The natural language text to extract data from
    /// * `additional_instructions` - Extra instructions to guide the extraction process
    ///
    /// # Returns
    ///
    /// A Result containing the extracted data as the specified type T
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use secretary::Task;
    /// # use secretary::llm_providers::openai::OpenAILLM;
    /// # use secretary::traits::GenerateData;
    /// # use serde::{Serialize, Deserialize};
    /// #
    /// # #[derive(Task, Serialize, Deserialize, Debug)]
    /// # struct ProductInfo {
    /// #     #[task(instruction = "Extract the product name")]
    /// #     pub name: String,
    /// #     #[task(instruction = "Extract price as a number")]
    /// #     pub price: f64,
    /// # }
    /// #
    /// # fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    /// // For reasoning models like o1-preview
    /// let llm = OpenAILLM::new(
    ///     "https://api.openai.com/v1",
    ///     "your-api-key",
    ///     "o1-preview"  // Reasoning model without JSON mode
    /// )?;
    ///
    /// let task = ProductInfo::new();
    /// let additional_instructions = vec!["Think step by step".to_string()];
    ///
    /// let input = "Apple MacBook Pro 16-inch costs $2,499";
    /// let result: ProductInfo = llm.force_generate_data(&task, input, &additional_instructions)?;
    /// # Ok(())
    /// # }
    /// ```
    fn force_generate_data<T: Task>(
        &self,
        task: &T,
        target: &str,
        additional_instructions: &Vec<String>,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let response: String = self.send_message(
            task.make_prompt(target, additional_instructions),
            false,
        )?;

        let value: Value = serde_json::from_str(&response).unwrap();
        let result: String = value["choices"][0]["message"]["content"]
            .as_str()
            .unwrap()
            .to_string();

        Ok(surfing::serde::from_mixed_text(&result)?)
    }
    
    /// Instead of generating a JSON in a single request. This method breaks a JSON down
    /// into single fields for the LLM to generate. It will eventually putting them together to make a final JSON. 
    fn fields_generate_data<T: Task>(
        &self,
        task: &T,
        target: &str,
        additional_instructions: &Vec<String>,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let messages: Vec<(String, Message)> = task.make_dstributed_generation_prompts(target, additional_instructions);
        
        let distributed_tasks_results: Vec<(String, String)> = std::thread::scope(|s|{
            let mut distributed_tasks = Vec::new();
            for (field_name, message) in messages {
                let handler = s.spawn(move || {
                    let result: String = self.send_message(message, false).unwrap();
                    (field_name, result)
                });
                
                distributed_tasks.push(handler);
            }
            
            let mut distributed_tasks_results: Vec<(String, String)> = Vec::new();
            for distributed_task in distributed_tasks {
                match distributed_task.join() {
                    Ok(result) => distributed_tasks_results.push(result),
                    Err(_) => panic!()
                }
            }
            
            distributed_tasks_results
        });
        
        dbg!(&distributed_tasks_results);
        Ok(generate_from_tuples!(T, distributed_tasks_results))
    }
}

/// Trait for asynchronous data generation from LLMs.
///
/// This trait provides async methods for extracting structured data from natural language text
/// using LLM providers. It includes both standard JSON mode generation and force generation
/// for reasoning models that don't support JSON mode.
///
/// # Examples
///
/// ```no_run
/// use secretary::Task;
/// use secretary::llm_providers::openai::OpenAILLM;
/// use secretary::traits::AsyncGenerateData;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Task, Debug, Serialize, Deserialize)]
/// struct ProductInfo {
///     #[task(instruction = "Extract the product name")]
///     pub name: String,
///     #[task(instruction = "Extract price as a number")]
///     pub price: f64,
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
///     let llm = OpenAILLM::new(
///         "https://api.openai.com/v1",
///         "your-api-key",
///         "gpt-4"
///     )?;
///     
///     let task = ProductInfo::new();
///     let additional_instructions = vec!["Be precise with pricing".to_string()];
///     
///     let input = "Apple MacBook Pro 16-inch costs $2,499";
///     let result: ProductInfo = llm.async_generate_data(&task, input, &additional_instructions).await?;
///     
///     println!("Extracted: {:#?}", result);
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait AsyncGenerateData
where
    Self: IsLLM,
{
    /// Asynchronously generates structured data from natural language using JSON mode.
    ///
    /// This is the asynchronous version of `generate_data` that can be used in async contexts.
    /// It uses the LLM's JSON mode (if available) to ensure structured output.
    ///
    /// # Arguments
    ///
    /// * `task` - A Task implementation that provides the system prompt and schema
    /// * `target` - The natural language text to extract data from
    /// * `additional_instructions` - Extra instructions to guide the extraction process
    ///
    /// # Returns
    ///
    /// A Result containing the extracted data as the specified type T
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The LLM API call fails
    /// - The response cannot be parsed as valid JSON
    /// - The JSON doesn't match the expected schema
    async fn async_generate_data<T: Task + Sync + Send>(
        &self,
        task: &T,
        target: &str,
        additional_instructions: &Vec<String>,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let request: Result<String, Box<dyn std::error::Error + Send + Sync>> = self
            .async_send_message(
                task.make_prompt(target, additional_instructions),
                true,
            )
            .await;

        let result = match request {
            Ok(result) => {
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

    /// Asynchronously generates structured data from natural language without JSON mode (for reasoning models).
    ///
    /// This is the asynchronous version of `force_generate_data` designed for reasoning models
    /// like o1, deepseek, and others that don't support JSON mode. It uses text parsing to
    /// extract JSON from the model's response.
    ///
    /// # Arguments
    ///
    /// * `task` - A Task implementation that provides the system prompt and schema
    /// * `target` - The natural language text to extract data from
    /// * `additional_instructions` - Extra instructions to guide the extraction process
    ///
    /// # Returns
    ///
    /// A Result containing the extracted data as the specified type T
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use secretary::Task;
    /// # use secretary::llm_providers::openai::OpenAILLM;
    /// # use secretary::traits::AsyncGenerateData;
    /// # use serde::{Serialize, Deserialize};
    /// #
    /// # #[derive(Task, Serialize, Deserialize, Debug)]
    /// # struct ProductInfo {
    /// #     #[task(instruction = "Extract the product name")]
    /// #     pub name: String,
    /// #     #[task(instruction = "Extract price as a number")]
    /// #     pub price: f64,
    /// # }
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    /// // For reasoning models like o1-preview
    /// let llm = OpenAILLM::new(
    ///     "https://api.openai.com/v1",
    ///     "your-api-key",
    ///     "o1-preview"  // Reasoning model without JSON mode
    /// )?;
    ///
    /// let task = ProductInfo::new();
    /// let additional_instructions = vec!["Think step by step".to_string()];
    ///
    /// let input = "Apple MacBook Pro 16-inch costs $2,499";
    /// let result: ProductInfo = llm.async_force_generate_data(&task, input, &additional_instructions).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn async_force_generate_data<T: Task + Sync + Send>(
        &self,
        task: &T,
        target: &str,
        additional_instructions: &Vec<String>,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let request: Result<String, Box<dyn std::error::Error + Send + Sync>> = self
            .async_send_message(
                task.make_prompt(target, additional_instructions),
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
