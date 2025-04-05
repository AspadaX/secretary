use std::{collections::HashMap, fmt::Display};

use anyhow::{anyhow, Error};
use async_openai::types::{ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestFunctionMessageArgs, ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestToolMessageArgs, ChatCompletionRequestUserMessageArgs, Role};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Prompt {
    data_structure: HashMap<String, String>,
    additional_instructions: Vec<String>,
    context: Vec<ChatCompletionRequestMessage>,
}

impl Prompt {
    /// Creates a new `Prompt` instance.
    ///
    /// # Arguments
    ///
    /// * `data_structure_with_annotations` - A data structure that can be serialized and deserialized.
    /// * `additional_instructions` - A string containing additional instructions.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use serde::{Deserialize, Serialize};
    /// 
    /// #[derive(Deserialize, Serialize)]
    /// struct School {
    ///     name: String,
    ///     kind: String,
    /// }
    ///
    /// let example = Example {
    ///     name: "A school name".to_string(),
    ///     kind: "mid-school, high-school, or elemenatry school".to_string(),
    /// };
    ///
    /// let prompt = Prompt::new(example, ["Categorize the text".to_string(), "John School is mid-school".to_string()]);
    /// ```
    pub fn new<'de, T>(data_structure_with_annotations: T, additional_instructions: Vec<String>) -> Self
    where
        T: Deserialize<'de> + Serialize,
    {
        let data_structure: HashMap<String, String> = serde_json::from_value(serde_json::to_value(data_structure_with_annotations).unwrap()).unwrap();
        Self {
            data_structure,
            additional_instructions,
            context: vec![]
        }
    }
    
    /// Update the context
    pub fn push(&mut self, role: Role, content: &str) -> Result<(), Error> {
        match role {
            Role::User => {
                self.context.push(
                    ChatCompletionRequestUserMessageArgs::default()
                        .content(content)
                        .build()
                        .unwrap()
                        .into()
                )
            },
            Role::Assistant => {
                self.context.push(
                    ChatCompletionRequestAssistantMessageArgs::default()
                        .content(content)
                        .build()
                        .unwrap()
                        .into()
                );
            },
            _ => return Err(anyhow!("Unsupported role"))
        }
        
        Ok(())
    }
    
    /// Get access right to read and write the context
    pub fn access_context(&mut self) -> &Vec<ChatCompletionRequestMessage> {
        &self.context
    }
}

impl Display for Prompt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut prompt = String::new();
        prompt.push_str("This is the json structure that you should strictly follow:\n");
        prompt.push_str(&serde_json::to_string(&self.data_structure).unwrap());
        prompt.push_str("\n");
        prompt.push_str("Besides, you should also following these instructions:\n");
        for additional_instruction in self.additional_instructions.iter() {
            prompt.push_str(
                &format!("- {}\n", additional_instruction)
            );
        }
        
        write!(f, "Respond in json.\n{}", prompt)
    }
}

impl Into<Vec<ChatCompletionRequestMessage>> for Prompt {
    fn into(self) -> Vec<ChatCompletionRequestMessage> {
        let mut final_context: Vec<ChatCompletionRequestMessage> = Vec::new();
        final_context.push(
            ChatCompletionRequestSystemMessageArgs::default()
                .content(self.to_string())
                .build()
                .unwrap()
                .into()
        );
        final_context.extend(self.context.clone());
        
        final_context
    }
}