#[derive(Debug, Serialize, Deserialize)]
struct OpenAICompatibleConfig {
    id: String,
    provider: String,
    api_key: String,
    api_url: String,
    model: String,
    max_tokens: i32,
    temperature: f32,
}