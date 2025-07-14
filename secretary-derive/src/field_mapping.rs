pub struct FieldMapping {
    pub field_name: String,
    pub instruction: String, 
    pub json_type: String,
}

impl FieldMapping {
    pub fn new(field_name: String, instruction: String, json_type: String) -> Self {
        Self {
            field_name,
            instruction,
            json_type
        }
    }
}
