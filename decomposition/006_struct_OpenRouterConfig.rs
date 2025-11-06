#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterConfig {
    id: String,
    provider: String,
    api_key: String,
    model: String,
    max_tokens: i32,
    temperature: f32,
}