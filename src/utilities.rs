/// Removes thinking blocks from LLM responses, particularly useful for reasoning models.
///
/// Many reasoning models (like o1-preview, deepseek-reasoner, etc.) wrap their internal
/// reasoning process in `<think></think>` tags. This function strips out these thinking
/// blocks to extract only the final answer or result.
///
/// # Arguments
///
/// * `content` - The raw response string from the LLM that may contain thinking blocks
///
/// # Returns
///
/// A cleaned string with all content between `<think>` and `</think>` tags removed
///
pub fn cleanup_thinking_blocks(content: String) -> String {
    let mut is_thinking: bool = false;
    let mut result: String = String::new();
    let mut first_line = true;

    for line in content.lines() {
        if line.trim() == "<think>" {
            is_thinking = true;
            continue;
        }

        if line.trim() == "</think>" {
            is_thinking = false;
            continue;
        }

        if !is_thinking {
            if !first_line {
                result.push('\n');
            }
            result.push_str(line);
            first_line = false;
        }
    }

    result
}

// Helper function to extract content from <result></result> tags
pub fn extract_result_content(content: &str) -> String {
    if let Some(start) = content.find("<result>") {
        if let Some(end) = content.find("</result>") {
            if start < end {
                return content[start + 8..end].trim().to_string();
            }
        }
    }
    content.trim().to_string()
}

/// Formats additional instructions into a structured prompt section.
///
/// This utility function takes a vector of instruction strings and formats them
/// into a readable bullet-point list that can be appended to LLM prompts.
/// If the vector is empty, it returns an empty string.
///
/// # Arguments
///
/// * `additional_instructions` - A vector of instruction strings to format
///
/// # Returns
///
/// A formatted string with instructions as bullet points, or empty string if no instructions
///
pub fn format_additional_instructions(additional_instructions: &Vec<String>) -> String {
    let mut prompt: String = String::new();
    // Add additional instructions
    if !additional_instructions.is_empty() {
        prompt.push_str("\nAdditional instructions:\n");
        for instruction in additional_instructions {
            prompt.push_str(&format!("- {}\n", instruction));
        }
    }

    prompt
}
