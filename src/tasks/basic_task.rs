use std::fmt::{Debug, Display};

use async_openai::types::ChatCompletionRequestMessage;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    message_list::{Message, MessageList},
    traits::{Context, SystemPrompt},
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BasicTask {
    data_structure: Value,
    additional_instructions: Vec<String>,
    context: MessageList,
}

impl BasicTask {
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
    ///     kind: "mid-school, high-school, or elementary school".to_string(),
    /// };
    ///
    /// let prompt = Prompt::new(example, ["Categorize the text".to_string(), "John School is mid-school".to_string()]);
    /// ```
    pub fn new<'de, T>(
        data_structure_with_annotations: T,
        additional_instructions: Vec<String>,
    ) -> Self
    where
        T: Deserialize<'de> + Serialize + Debug,
    {
        let data_structure: Value = serde_json::to_value(data_structure_with_annotations).unwrap();
        Self {
            data_structure,
            additional_instructions,
            context: MessageList::new(),
        }
    }
}

impl SystemPrompt for BasicTask {
    fn get_system_prompt(&self) -> String {
        let mut prompt = String::new();
        prompt.push_str("This is the json structure that you should strictly follow:\n");
        prompt.push_str(&serde_json::to_string(&self.data_structure).unwrap());
        prompt.push_str("\n");
        prompt.push_str("Besides, you should also following these instructions:\n");
        for additional_instruction in self.additional_instructions.iter() {
            prompt.push_str(&format!("- {}\n", additional_instruction));
        }

        prompt
    }
}

impl Display for BasicTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(&self.data_structure).unwrap()
        )
    }
}

impl Into<Vec<ChatCompletionRequestMessage>> for BasicTask {
    fn into(self) -> Vec<ChatCompletionRequestMessage> {
        self.get_context().into()
    }
}

impl Context for BasicTask {
    fn get_context_mut(&mut self) -> &mut crate::message_list::MessageList {
        &mut self.context
    }

    fn get_context(&self) -> MessageList {
        let mut final_context: MessageList = MessageList::new();
        final_context.push(Message::new(
            crate::message_list::Role::System,
            self.get_system_prompt(),
        ));
        
        final_context.extend(self.context.clone());
        
        final_context
    }
}
