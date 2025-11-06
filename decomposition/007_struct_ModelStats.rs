#[derive(Debug, Serialize, Deserialize, Clone)]
struct ModelStats {
    success_count: usize,
    failure_count: usize,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    last_success: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    last_failure: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    jail_until: Option<chrono::DateTime<chrono::Utc>>,
    jail_count: usize,
    blacklisted: bool,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    blacklisted_since: Option<chrono::DateTime<chrono::Utc>>,
}