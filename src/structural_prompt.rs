use std::{collections::HashMap, fmt::Display};

use anyhow::{anyhow, Error};
use async_openai::types::{ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestFunctionMessageArgs, ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestToolMessageArgs, ChatCompletionRequestUserMessageArgs, Role};
use serde::{Deserialize, Serialize};

use crate::traits::Context;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StructuralPrompt {
    data_structure: HashMap<String, String>,
    additional_instructions: Vec<String>,
    context: Vec<ChatCompletionRequestMessage>,
}

impl StructuralPrompt {
    /// TODO: docs and examples
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
}

impl Context for StructuralPrompt {
    fn get_context_mut(&mut self) -> &mut Vec<ChatCompletionRequestMessage> {
        &mut self.context
    }
}

impl Display for StructuralPrompt {
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

impl Into<Vec<ChatCompletionRequestMessage>> for StructuralPrompt {
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