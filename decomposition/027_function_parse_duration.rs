fn parse_duration(duration_str: &str) -> Result<std::time::Duration, String> {
    let duration_str = duration_str.trim().to_lowercase();
    if duration_str.is_empty() {
        return Err("Duration string is empty".to_string());
    }

    let mut chars = duration_str.chars().peekable();
    let mut number = String::new();
    
    // Collect digits
    while let Some(c) = chars.peek() {
        if c.is_digit(10) {
            number.push(chars.next().unwrap());
        } else {
            break;
        }
    }

    // Get the unit (rest of the string)
    let unit: String = chars.collect();

    if number.is_empty() {
        return Err("No duration value provided".to_string());
    }

    let value = number.parse::<u64>()
        .map_err(|_| format!("Invalid duration number: {}", number))?;

    match unit.as_str() {
        "s" => Ok(std::time::Duration::from_secs(value)),
        "m" => Ok(std::time::Duration::from_secs(value * 60)),
        "h" => Ok(std::time::Duration::from_secs(value * 3600)),
        _ => Err(format!("Invalid duration unit: '{}'. Use s, m, or h", unit)),
    }
}