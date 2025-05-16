use std::{collections::HashMap, fmt::Display};

use anyhow::{Error, anyhow};
use async_openai::types::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestFunctionMessageArgs,
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestToolMessageArgs, ChatCompletionRequestUserMessageArgs, Role,
};
use serde::{Deserialize, Serialize};

use crate::traits::Context;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContextualPromptStructure {
    reasoning: String,
    content: Option<String>,
    notes: Vec<String>,
    data_structure: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContextualPrompt {
    contextual_prompt_structure: ContextualPromptStructure,
}

impl Default for ContextualPromptStructure {
    fn default() -> Self {
        Self { 
            reasoning: "your thoughts on your response.".to_string(), 
            content: Some("anything that you would like to ask the user. leave it to null if you had collected all the checklist items".to_string()),
            notes: vec!["keypoints that you think are helpful to reach the final conclusion. append only.".to_string()], 
            data_structure: HashMap::new()
        }
    }
}

impl ContextualPrompt {
    /// Creates a new `ContextualPrompt` from a given data structure.
    ///
    /// This method converts the provided data structure into a HashMap<String, String> and initializes
    /// a new `ContextualPrompt` with an empty context.
    ///
    /// # Arguments
    ///
    /// * `data_structure_with_annotations` - A data structure that can be serialized to and deserialized from JSON
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use your_crate::ContextualPrompt;
    ///
    /// #[derive(serde::Serialize, serde::Deserialize)]
    /// struct MyStruct {
    ///     field1: String,
    ///     field2: i32,
    /// }
    ///
    /// let data = MyStruct {
    ///     field1: "value".to_string(),
    ///     field2: 42,
    /// };
    ///
    /// let prompt = ContextualPrompt::new(data);
    /// ```
    pub fn new<'de, T>(data_structure_with_annotations: T) -> Self
    where
        T: Deserialize<'de> + Serialize,
    {
        let data_structure: HashMap<String, String> =
            serde_json::from_value(serde_json::to_value(data_structure_with_annotations).unwrap())
                .unwrap();
        Self {
            contextual_prompt_structure: ContextualPromptStructure {
                reasoning: String::new(),
                content: None,
                notes: Vec::new(),
                data_structure,
            },
        }
    }
}

impl Context for ContextualPrompt {
    fn get_context_mut(&mut self) -> &mut crate::message_list::MessageList {
        let mut messages = Vec::new();
        for note in self.contextual_prompt_structure.notes {
            
        }
    }
}

impl Display for ContextualPrompt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut prompt = String::new();
        prompt.push_str("This is the json structure that you should strictly follow:\n");
        prompt.push_str(&serde_json::to_string(&self.data_structure).unwrap());
        prompt.push_str("\n");
        prompt.push_str("Besides, you should also following these instructions:\n");
        for additional_instruction in self.additional_instructions.iter() {
            prompt.push_str(&format!("- {}\n", additional_instruction));
        }

        write!(f, "Respond in json.\n{}", prompt)
    }
}

impl Into<Vec<ChatCompletionRequestMessage>> for ContextualPrompt {
    fn into(self) -> Vec<ChatCompletionRequestMessage> {
        let mut final_context: Vec<ChatCompletionRequestMessage> = Vec::new();
        final_context.push(
            ChatCompletionRequestSystemMessageArgs::default()
                .content(self.to_string())
                .build()
                .unwrap()
                .into(),
        );
        final_context.extend(self.context.clone());

        final_context
    }
}
