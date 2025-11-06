#[derive(Debug, Serialize, Deserialize)]
struct Config {
    providers: Vec<ProviderConfig>,
    active_provider: String,
    #[serde(default = "default_retry_attempts")]
    retry_attempts: u32,
}