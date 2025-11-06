fn is_model_available(model_stats: &Option<&ModelStats>) -> bool {
    match model_stats {
        None => true, // No stats yet, model is available
        Some(stats) => {
            // Check if blacklisted but should be retried
            if stats.blacklisted {
                if let Some(blacklisted_since) = stats.blacklisted_since {
                    let retry_duration = chrono::Duration::days(BLACKLIST_RETRY_DAYS);
                    let now = chrono::Utc::now();
                    
                    // If blacklisted for more than retry period, give it another chance
                    if now - blacklisted_since > retry_duration {
                        return true;
                    }
                    return false;
                }
                return false;
            }
            
            // Check if currently in jail
            if let Some(jail_until) = stats.jail_until {
                if chrono::Utc::now() < jail_until {
                    return false;
                }
            }
            
            true
        }
    }
}