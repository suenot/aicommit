#[derive(Debug, Deserialize)]
struct OpenRouterUsage {
    prompt_tokens: i32,
    completion_tokens: i32,
    total_tokens: i32,
}