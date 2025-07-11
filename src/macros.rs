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

        // Helper function to intelligently parse and clean values based on common patterns
        fn smart_parse_value(content: &str) -> Value {
            let cleaned = content.trim();
            
            // Handle empty or null-like values
            if cleaned.is_empty() || cleaned.eq_ignore_ascii_case("null") || cleaned.eq_ignore_ascii_case("none") {
                return Value::Null;
            }
            
            // Handle boolean values (case-insensitive)
            if cleaned.eq_ignore_ascii_case("true") {
                return Value::Bool(true);
            }
            if cleaned.eq_ignore_ascii_case("false") {
                return Value::Bool(false);
            }
            
            // Handle numeric values with currency symbols, commas, and other formatting
            if let Some(numeric_value) = parse_numeric_value(cleaned) {
                // Check if it's a whole number (integer)
                if numeric_value.fract() == 0.0 && numeric_value >= 0.0 && numeric_value <= u64::MAX as f64 {
                    // Use integer representation for whole numbers
                    return Value::Number(serde_json::Number::from(numeric_value as u64));
                } else {
                    // Use floating point for decimals
                    return Value::Number(serde_json::Number::from_f64(numeric_value).unwrap_or_else(|| serde_json::Number::from(0)));
                }
            }
            
            // Try parsing as JSON first (for arrays, objects, quoted strings)
            if let Ok(json_value) = serde_json::from_str::<Value>(cleaned) {
                return json_value;
            }
            
            // Default to string value
            Value::String(cleaned.to_string())
        }
        
        // Helper function to parse numeric values with various formatting
        fn parse_numeric_value(content: &str) -> Option<f64> {
            let mut cleaned = content.to_string();
            
            // Remove common currency symbols
            cleaned = cleaned.replace('$', "");
            cleaned = cleaned.replace('€', "");
            cleaned = cleaned.replace('£', "");
            cleaned = cleaned.replace('¥', "");
            cleaned = cleaned.replace('₹', "");
            
            // Remove commas (thousand separators)
            cleaned = cleaned.replace(',', "");
            
            // Remove spaces
            cleaned = cleaned.replace(' ', "");
            
            // Handle percentage
            let is_percentage = cleaned.ends_with('%');
            if is_percentage {
                cleaned = cleaned.trim_end_matches('%').to_string();
            }
            
            // Try to parse as float
            if let Ok(mut num) = cleaned.parse::<f64>() {
                if is_percentage {
                    num /= 100.0; // Convert percentage to decimal
                }
                return Some(num);
            }
            
            None
        }

        // Helper function to set nested field values
        fn set_nested_field(json_map: &mut Map<String, Value>, field_path: &str, value: Value) {
            let parts: Vec<&str> = field_path.split('.').collect();
            
            if parts.len() == 1 {
                // Simple field, set directly
                json_map.insert(parts[0].to_string(), value);
            } else {
                // Nested field, create nested structure
                let first_part = parts[0];
                let remaining_path = parts[1..].join(".");
                
                // Get or create the nested object
                let nested_obj = json_map.entry(first_part.to_string())
                    .or_insert_with(|| Value::Object(Map::new()));
                
                if let Value::Object(nested_map) = nested_obj {
                    set_nested_field(nested_map, &remaining_path, value);
                }
            }
        }

        // Create a JSON object from the field tuples
        let mut json_map = Map::new();

        for (field_name, content) in $tuples {
            // Extract content from <result></result> tags if present
            let cleaned_content = extract_result_content(&content);

            // Use smart parsing to handle various data types and formats
            let value = smart_parse_value(&cleaned_content);

            // Handle nested field paths
            set_nested_field(&mut json_map, &field_name, value);
        }

        // Convert the JSON object to the target type
        let json_value = Value::Object(json_map);
        serde_json::from_value::<$obj_type>(json_value).unwrap_or_else(|_| <$obj_type>::default())
    }};
}
