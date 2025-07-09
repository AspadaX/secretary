use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedGenerationPrompt {
    pub field_name: String,
    pub prompt: String,
}