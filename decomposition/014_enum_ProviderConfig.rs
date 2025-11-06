#[derive(Debug, Serialize, Deserialize)]
enum ProviderConfig {
    OpenRouter(OpenRouterConfig),
    Ollama(OllamaConfig),
    OpenAICompatible(OpenAICompatibleConfig),
    SimpleFreeOpenRouter(SimpleFreeOpenRouterConfig),
    ClaudeCode(ClaudeCodeConfig),
    OpenCode(OpenCodeConfig),
}