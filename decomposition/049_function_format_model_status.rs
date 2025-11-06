fn format_model_status(model: &str, stats: &ModelStats) -> String {
    let status = if stats.blacklisted {
        "BLACKLISTED".to_string()
    } else if let Some(jail_until) = stats.jail_until {
        if chrono::Utc::now() < jail_until {
            let remaining = jail_until - chrono::Utc::now();
            format!("JAILED ({}h remaining)", remaining.num_hours())
        } else {
            "ACTIVE".to_string()
        }
    } else {
        "ACTIVE".to_string()
    };
    
    let last_success = stats.last_success.map_or("Never".to_string(), |ts| {
        let ago = chrono::Utc::now() - ts;
        if ago.num_days() > 0 {
            format!("{} days ago", ago.num_days())
        } else if ago.num_hours() > 0 {
            format!("{} hours ago", ago.num_hours())
        } else {
            format!("{} minutes ago", ago.num_minutes())
        }
    });
    
    let last_failure = stats.last_failure.map_or("Never".to_string(), |ts| {
        let ago = chrono::Utc::now() - ts;
        if ago.num_days() > 0 {
            format!("{} days ago", ago.num_days())
        } else if ago.num_hours() > 0 {
            format!("{} hours ago", ago.num_hours())
        } else {
            format!("{} minutes ago", ago.num_minutes())
        }
    });
    
    format!("{}: {} (Success: {}, Failure: {}, Last success: {}, Last failure: {})",
            model, status, stats.success_count, stats.failure_count, last_success, last_failure)
}