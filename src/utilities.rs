/// Clean up reasoning models' thinking blocks
pub fn cleanup_thinking_blocks(content: String) -> String {
    let mut is_thinking: bool = false;
    let mut result: String = String::new();
    for line in content.lines() {
        if !is_thinking {
            result.push_str(line);
        }
        
        if line.trim() == "<think>" {
            is_thinking = true;
            continue;
        }
        
        if line.trim() == "</think>" {
            is_thinking = false;
            continue;
        }
        
    }
    
    result
}