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

        // Helper function to intelligently parse and clean values based on common patterns
        fn smart_parse_value(content: &str, field_name: &str) -> Value {
            let cleaned = content.trim();

            // Handle empty or null-like values
            if cleaned.is_empty() || cleaned.eq_ignore_ascii_case("null") || cleaned.eq_ignore_ascii_case("none") {
                return Value::Null;
            }

            // Try parsing as JSON first (for arrays, objects, quoted strings)
            // This is more robust as it handles cases where LLM returns JSON strings
            if let Ok(json_value) = serde_json::from_str::<Value>(cleaned) {
                // If it's a JSON object with a single key that matches the field name,
                // extract the inner value (common LLM response pattern)
                if let Value::Object(obj) = &json_value {
                    let field_key = field_name.split('.').last().unwrap_or(field_name);
                    if obj.len() == 1 && obj.contains_key(field_key) {
                        return obj[field_key].clone();
                    }
                }

                return json_value;
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

                return;
            }

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

        // Create a JSON object from the field tuples
        let mut json_map = Map::new();

        for (field_name, content) in $tuples {
            // Use smart parsing to handle various data types and formats
            let value = smart_parse_value(&content, &field_name);

            // Handle nested field paths
            set_nested_field(&mut json_map, &field_name, value);
        }

        // Convert the JSON object to the target type
        let json_value = Value::Object(json_map);

        // First attempt full deserialization
        match serde_json::from_value::<$obj_type>(json_value.clone()) {
            Ok(result) => result,
            Err(original_error) => {
                // If full deserialization fails, perform field-by-field validation
                let mut failed_fields = Vec::new();
                let mut successful_fields = Vec::new();

                if let Value::Object(ref map) = json_value {
                    // Create a default instance to test field compatibility
                    let default_instance = <$obj_type>::default();
                    let default_json = serde_json::to_value(&default_instance).unwrap_or(Value::Object(serde_json::Map::new()));

                    if let Value::Object(default_map) = default_json {
                        for (field_name, field_value) in map {
                            // Check if this field exists in the target struct
                            if default_map.contains_key(field_name) {
                                // Create a test object with default values but this specific field
                                let mut test_map = default_map.clone();
                                test_map.insert(field_name.clone(), field_value.clone());
                                let test_json = Value::Object(test_map);

                                match serde_json::from_value::<$obj_type>(test_json) {
                                    Ok(_) => successful_fields.push(field_name.clone()),
                                    Err(_) => failed_fields.push(field_name.clone()),
                                }
                            } else {
                                // Field doesn't exist in target struct
                                failed_fields.push(field_name.clone());
                            }
                        }
                    }
                }

                // If we have field-level information, create detailed error
                if !failed_fields.is_empty() {
                    use crate::error::FieldDeserializationError;
                    panic!(
                        "{}",
                        FieldDeserializationError {
                            failed_fields,
                            successful_fields,
                            original_error: original_error.to_string(),
                        }
                    );
                }

                // Fallback to default if no specific field errors identified
                <$obj_type>::default()
            }
        }
    }};
}
