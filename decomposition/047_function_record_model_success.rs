fn record_model_success(model_stats: &mut ModelStats) {
    model_stats.success_count += 1;
    model_stats.last_success = Some(chrono::Utc::now());
    
    // Reset consecutive failures if successful
    if model_stats.last_failure.is_none() || 
       model_stats.last_success.unwrap() > model_stats.last_failure.unwrap() {
        // The model is working now, remove any jail time
        model_stats.jail_until = None;
    }
}