#[derive(Debug, Serialize, Deserialize, Clone)]
struct SimpleFreeOpenRouterConfig {
    id: String,
    provider: String,
    api_key: String,
    max_tokens: i32,
    temperature: f32,
    #[serde(default)]
    failed_models: Vec<String>,
    #[serde(default)]
    model_stats: std::collections::HashMap<String, ModelStats>,
    #[serde(default)]
    last_used_model: Option<String>,
    #[serde(default = "chrono::Utc::now")]
    last_config_update: chrono::DateTime<chrono::Utc>,
}