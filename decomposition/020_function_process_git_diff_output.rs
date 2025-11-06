fn process_git_diff_output(diff: &str) -> String {
    // Early return if diff is small enough
    if diff.len() <= MAX_DIFF_CHARS {
        return diff.to_string();
    }
    
    // Split the diff into file sections
    let file_pattern = r"(?m)^diff --git ";
    let sections: Vec<&str> = diff.split(file_pattern).collect();
    
    // First segment is empty or small header - keep it as is
    let mut processed = if !sections.is_empty() && !sections[0].trim().is_empty() {
        sections[0].to_string()
    } else {
        String::new()
    };
    
    // Process each file section
    for (i, section) in sections.iter().enumerate().skip(1) {
        // Skip empty sections
        if section.trim().is_empty() {
            continue;
        }
        
        // Add the "diff --git " prefix back (we split on it, so it's missing)
        processed.push_str("diff --git ");
        
        // Check if this section is too large
        if section.len() > MAX_FILE_DIFF_CHARS {
            // Find the file name from the section
            let _file_name = if let Some(end) = section.find('\n') {
                section[..end].trim()
            } else {
                section.trim()
            };
            
            // Take the header part (usually 4-5 lines) - this includes the file name, index, and --- +++ lines
            let _header_end = section.lines().take(5).collect::<Vec<&str>>().join("\n").len();
            
            // Take the beginning of the diff content - safely truncate at char boundary
            let safe_len = get_safe_slice_length(section, MAX_FILE_DIFF_CHARS.min(section.len()));
            let content = &section[..safe_len];
            
            // Add truncation notice for this specific file
            processed.push_str(&format!("{}\n\n[... diff for this file truncated due to length ...]", content));
        } else {
            // Section is small enough, add it as is
            processed.push_str(section);
        }
        
        // Add separating newline if needed
        if i < sections.len() - 1 && !processed.ends_with('\n') {
            processed.push('\n');
        }
    }
    
    // Final overall truncation check (as a safety measure)
    if processed.len() > MAX_DIFF_CHARS {
        let safe_len = get_safe_slice_length(&processed, MAX_DIFF_CHARS - 100);
        processed = format!("{}...\n\n[... total diff truncated due to length (first {} chars shown) ...]", 
            &processed[..safe_len], 
            safe_len);
    }
    
    processed
}