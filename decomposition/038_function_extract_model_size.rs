fn extract_model_size(model_name: &str) -> u32 {
    let lower_name = model_name.to_lowercase();
    
    // Look for patterns like "70b", "32b", "7b", etc.
    let patterns = [
        "253b", "235b", "200b", "124b",
        "70b", "80b", "90b", "72b", "65b", 
        "40b", "32b", "30b", "24b", "20b",
        "16b", "14b", "13b", "12b", "11b", "10b",
        "9b", "8b", "7b", "6b", "5b", "4b", "3b", "2b", "1b"
    ];
    
    for pattern in patterns {
        if lower_name.contains(pattern) {
            // Extract the number from the pattern (e.g., "70b" -> 70)
            if let Ok(size) = pattern.trim_end_matches(|c| c == 'b' || c == 'B').parse::<u32>() {
                return size;
            }
        }
    }
    
    // Default size if no pattern matches
    // Check for specific keywords that might indicate a more powerful model
    if lower_name.contains("large") || lower_name.contains("ultra") {
        return 15; // Assume it's a medium-large model
    } else if lower_name.contains("medium") {
        return 10;
    } else if lower_name.contains("small") || lower_name.contains("tiny") {
        return 5;
    }
    
    // Default fallback
    0
}