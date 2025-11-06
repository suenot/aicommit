fn record_model_failure(model_stats: &mut ModelStats) {
    let now = chrono::Utc::now();
    model_stats.failure_count += 1;
    model_stats.last_failure = Some(now);
    
    // Check if we have consecutive failures
    let has_consecutive_failures = match model_stats.last_success {
        None => true, // Never had a success
        Some(last_success) => {
            // If last success is older than last failure, we have consecutive failures
            model_stats.last_failure.unwrap() > last_success
        }
    };
    
    if has_consecutive_failures {
        // Count consecutive failures by comparing timestamps
        let consecutive_failures = if let Some(last_success) = model_stats.last_success {
            let hours_since_success = (now - last_success).num_hours();
            // If it's been more than a day since last success, count as consecutive failures
            if hours_since_success > 24 {
                model_stats.failure_count.min(MAX_CONSECUTIVE_FAILURES)
            } else {
                // Count failures since last success
                1 // This is at least 1 consecutive failure
            }
        } else {
            // No success ever, count all failures
            model_stats.failure_count.min(MAX_CONSECUTIVE_FAILURES)
        };
        
        // Jail if we hit the threshold
        if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
            // Calculate jail duration based on recidivism
            let jail_hours = INITIAL_JAIL_HOURS * JAIL_TIME_MULTIPLIER.pow(model_stats.jail_count as u32);
            let jail_hours = jail_hours.min(MAX_JAIL_HOURS); // Cap at maximum
            
            // Set jail expiration time
            model_stats.jail_until = Some(now + chrono::Duration::hours(jail_hours));
            model_stats.jail_count += 1;
            
            // Blacklist if consistently problematic
            if model_stats.jail_count >= BLACKLIST_AFTER_JAIL_COUNT {
                model_stats.blacklisted = true;
                model_stats.blacklisted_since = Some(now);
            }
        }
    }
}