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
        use crate::utilities::json::{smart_parse_value, set_nested_field};

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
