/// Macro that generates an object by setting its fields from tuples of field names and values.
/// This macro uses serde_json to deserialize field values from the LLM responses.
///
/// # Arguments
///
/// * `obj_type` - The type of object to create
/// * `tuples` - A vector of tuples where each tuple contains a field name and the content for that field
#[macro_export]
macro_rules! generate_from_tuples {
    ($obj_type:ty, $tuples:expr) => {{
        use serde_json::{Map, Value};

        // Helper function to extract content from <result></result> tags
        fn extract_result_content(content: &str) -> String {
            if let Some(start) = content.find("<result>") {
                if let Some(end) = content.find("</result>") {
                    if start < end {
                        return content[start + 8..end].trim().to_string();
                    }
                }
            }
            content.trim().to_string()
        }

        // Create a JSON object from the field tuples
        let mut json_map = Map::new();

        for (field_name, content) in $tuples {
            // Extract content from <result></result> tags if present
            let cleaned_content = extract_result_content(&content);

            // Try to parse as JSON first, fallback to string
            let value = match serde_json::from_str::<Value>(&cleaned_content) {
                Ok(v) => v,
                Err(_) => Value::String(cleaned_content),
            };

            json_map.insert(field_name, value);
        }

        // Convert the JSON object to the target type
        let json_value = Value::Object(json_map);
        serde_json::from_value::<$obj_type>(json_value).unwrap_or_else(|_| <$obj_type>::default())
    }};
}
