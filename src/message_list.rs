use async_openai::types::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestAssistantMessageContent,
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestSystemMessageContent, ChatCompletionRequestUserMessageArgs,
    ChatCompletionRequestUserMessageContent,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    #[default]
    User,
    Assistant,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    pub fn new(role: Role, content: String) -> Self {
        Message { role, content }
    }
}

impl From<ChatCompletionRequestMessage> for Message {
    fn from(value: ChatCompletionRequestMessage) -> Self {
        match value {
            ChatCompletionRequestMessage::System(system_msg) => Message {
                role: Role::System,
                content: match system_msg.content {
                    ChatCompletionRequestSystemMessageContent::Text(text) => text,
                    ChatCompletionRequestSystemMessageContent::Array(_) => String::new(),
                },
            },
            ChatCompletionRequestMessage::User(user_msg) => Message {
                role: Role::User,
                content: match user_msg.content {
                    ChatCompletionRequestUserMessageContent::Text(text) => text,
                    ChatCompletionRequestUserMessageContent::Array(_) => String::new(),
                },
            },
            ChatCompletionRequestMessage::Assistant(assistant_msg) => Message {
                role: Role::Assistant,
                content: match assistant_msg.content {
                    Some(ChatCompletionRequestAssistantMessageContent::Text(text)) => text,
                    Some(ChatCompletionRequestAssistantMessageContent::Array(_)) => String::new(),
                    None => String::new(),
                },
            },
            _ => Message {
                role: Role::User,
                content: String::new(),
            },
        }
    }
}

impl From<Message> for ChatCompletionRequestMessage {
    fn from(message: Message) -> Self {
        match message.role {
            Role::System => ChatCompletionRequestSystemMessageArgs::default()
                .content(message.content)
                .build()
                .unwrap()
                .into(),
            Role::User => ChatCompletionRequestUserMessageArgs::default()
                .content(message.content)
                .build()
                .unwrap()
                .into(),
            Role::Assistant => ChatCompletionRequestAssistantMessageArgs::default()
                .content(message.content)
                .build()
                .unwrap()
                .into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct MessageList {
    messages: Vec<Message>,
}

impl MessageList {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    pub fn push(&mut self, message: Message) {
        self.messages.push(message);
    }
}

impl IntoIterator for MessageList {
    type Item = Message;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.messages.into_iter()
    }
}

impl Extend<Message> for MessageList {
    fn extend<T: IntoIterator<Item = Message>>(&mut self, iter: T) {
        self.messages.extend(iter);
    }
}

impl From<Vec<ChatCompletionRequestMessage>> for MessageList {
    fn from(value: Vec<ChatCompletionRequestMessage>) -> Self {
        let mut messages: Vec<Message> = Vec::new();
        for message in value {
            messages.push(Message::from(message));
        }

        Self { messages }
    }
}

impl Into<Vec<ChatCompletionRequestMessage>> for MessageList {
    fn into(self) -> Vec<ChatCompletionRequestMessage> {
        let mut messages: Vec<ChatCompletionRequestMessage> = Vec::new();
        for message in self.messages {
            messages.push(ChatCompletionRequestMessage::from(message));
        }

        messages
    }
}
