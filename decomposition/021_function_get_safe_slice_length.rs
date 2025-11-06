fn get_safe_slice_length(s: &str, max_len: usize) -> usize {
    if max_len >= s.len() {
        return s.len();
    }
    
    // Find the largest index <= max_len that is a char boundary
    let mut safe_len = max_len;
    while safe_len > 0 && !s.is_char_boundary(safe_len) {
        safe_len -= 1;
    }
    
    safe_len
}