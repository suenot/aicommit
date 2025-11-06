impl Default for ModelStats {
    fn default() -> Self {
        Self {
            success_count: 0,
            failure_count: 0,
            last_success: None,
            last_failure: None,
            jail_until: None,
            jail_count: 0,
            blacklisted: false,
            blacklisted_since: None,
        }
    }
}