use std::fmt::Display;

#[derive(Debug)]
pub enum ValidationError {
    PrimitiveMissingInstruction(String),
    TaskFieldWithInstruction(String),
    UnknownFieldType(String),
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            ValidationError::PrimitiveMissingInstruction(field) => write!(f, "Missing instruction for primitive field: {}", field),
            ValidationError::TaskFieldWithInstruction(field) => write!(f, "Task field with instruction: {}", field),
            ValidationError::UnknownFieldType(field) => write!(f, "Unknown field type: {}", field),
        }
    }
}

impl std::error::Error for ValidationError {}