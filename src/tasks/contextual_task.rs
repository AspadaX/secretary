use std::{collections::HashSet, fmt::Display};

use async_openai::types::ChatCompletionRequestMessage;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    message_list::{Message, MessageList, Role},
    traits::{Context, FromJSON, SystemPrompt, ToJSON},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContextualTaskPromptDataStructure {
    reasoning: String,
    content: Option<String>,
    notes: HashSet<String>,
    data_structure: Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContextualTask {
    contextual_task_prompt_structure: ContextualTaskPromptDataStructure,
    additional_instructions: Vec<String>,
    context_state: ContextualTaskPromptDataStructure,
    context: MessageList,
}

impl Default for ContextualTaskPromptDataStructure {
    fn default() -> Self {
        let mut notes: HashSet<String> = HashSet::new();
        notes.insert("Summaries of the user's query and your response, as well as any observations you found.".to_string());

        Self {
            reasoning: "Your thoughts on your response.".to_string(), 
            content: Some("Anything that you would like to ask the user. leave it to null if you had collected all the items".to_string()),
            notes,
            data_structure: Value::default(),
        }
    }
}

impl ContextualTask {
    /// Creates a new `ContextualTask` from a given data structure.
    ///
    /// This method converts the provided data structure into a HashMap<String, String> and initializes
    /// a new `ContextualTask` with an empty context.
    ///
    /// # Arguments
    ///
    /// * `data_structure_with_annotations` - A data structure that can be serialized to and deserialized from JSON
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use your_crate::ContextualTask;
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
    /// let prompt = ContextualTask::new(data);
    /// ```
    pub fn new<'de, T>(
        data_structure_with_annotations: T,
        additional_instructions: Vec<String>,
    ) -> Self
    where
        T: Deserialize<'de> + Serialize,
    {
        let data_structure: Value = serde_json::to_value(data_structure_with_annotations).unwrap();
        Self {
            contextual_task_prompt_structure: ContextualTaskPromptDataStructure {
                data_structure,
                ..Default::default()
            },
            additional_instructions,
            context_state: ContextualTaskPromptDataStructure::default(),
            context: MessageList::new(),
        }
    }
    
    /// Cleaup a role's message in the chat history
    pub fn cleanup_messages_by_role(&mut self, role: Role) {
        let mut messages_index: Vec<usize> = Vec::new();
        for (index, message) in self.context.iter().enumerate() {
            if message.role == role {
                messages_index.push(index);
            }
        }
        
        for index in messages_index {
            self.context.remove(index);
        }
    }
    
    pub fn update_context_state(&mut self, contextual_task_prompt: &ContextualTaskPromptDataStructure) {
        self.context_state.reasoning = contextual_task_prompt.reasoning.clone();
        self.context_state.content = contextual_task_prompt.content.clone();
        self.context_state.data_structure = contextual_task_prompt.data_structure.clone();

        // Extend notes with the new notes
        for note in &contextual_task_prompt.notes {
            self.context_state.notes.insert(note.clone());
        } 
    }
}

impl SystemPrompt for ContextualTask {
    fn get_system_prompt(&self) -> String {
        let mut prompt = String::new();
        prompt.push_str("This is the json structure that you should strictly follow:\n");
        prompt.push_str(&serde_json::to_string(&self.contextual_task_prompt_structure).unwrap());
        prompt.push_str("\n");
        prompt.push_str("Besides, you should also following these instructions:\n");
        for additional_instruction in self.additional_instructions.iter() {
            prompt.push_str(&format!("- {}\n", additional_instruction));
        }

        format!("Respond in json.\n{}", prompt)
    }
}

impl Display for ContextualTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(&self.contextual_task_prompt_structure).unwrap()
        )
    }
}

impl Into<Vec<ChatCompletionRequestMessage>> for ContextualTask {
    fn into(self) -> Vec<ChatCompletionRequestMessage> {
        self.get_context().into()
    }
}

impl Context for ContextualTask {
    fn get_context_mut(&mut self) -> &mut MessageList {
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
    
    fn push(&mut self, role: crate::message_list::Role, content: &str) -> anyhow::Result<(), anyhow::Error> {
        match role {
            Role::Assistant => {
                let assistant_response: ContextualTaskPromptDataStructure = serde_json::from_str(content).unwrap();
                self.update_context_state(&assistant_response);
                
                self.cleanup_messages_by_role(role);
                self.context.push(
                    Message::new(
                        Role::Assistant, serde_json::to_string_pretty(&self.context_state).unwrap()
                    )
                );
            },
            Role::User => {
                self.cleanup_messages_by_role(role);
                self.context.push(Message::new(role, content.to_string()));
            },
            Role::System => self.context.push(Message::new(role, content.to_string())),
        }
        
        Ok(())
    }
}

impl ToJSON for ContextualTask {}

impl FromJSON for ContextualTask {}
