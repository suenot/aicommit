const MAX_DIFF_CHARS: usize = 15000; // Limit diff size to prevent excessive API usage
const MAX_FILE_DIFF_CHARS: usize = 3000; // Maximum characters per file diff section

// Define a list of preferred free models from best to worst
const PREFERRED_FREE_MODELS: &[&str] = &[
    // Meta models - Llama 4 series
    "meta-llama/llama-4-maverick:free",
    "meta-llama/llama-4-scout:free",
    
    // Ultra large models (200B+)
    "nvidia/llama-3.1-nemotron-ultra-253b-v1:free",
    "qwen/qwen3-235b-a22b:free",
    
    // Very large models (70B-200B)
    "meta-llama/llama-3.1-405b:free",
    "nvidia/llama-3.3-nemotron-super-49b-v1:free",
    "meta-llama/llama-3.3-70b-instruct:free",
    "deepseek/deepseek-r1-distill-llama-70b:free",
    "shisa-ai/shisa-v2-llama3.3-70b:free",
    
    // Large models (32B-70B)
    "qwen/qwen-2.5-72b-instruct:free",
    "qwen/qwen2.5-vl-72b-instruct:free",
    "bytedance-research/ui-tars-72b:free",
    "featherless/qwerky-72b:free",
    "thudm/glm-4-32b:free",
    "thudm/glm-z1-32b:free",
    "qwen/qwen3-32b:free",
    "qwen/qwen3-30b-a3b:free",
    "qwen/qwq-32b:free",
    "qwen/qwq-32b-preview:free",
    "deepseek/deepseek-r1-distill-qwen-32b:free",
    "arliai/qwq-32b-arliai-rpr-v1:free",
    "qwen/qwen2.5-vl-32b-instruct:free",
    "open-r1/olympiccoder-32b:free",
    "qwen/qwen-2.5-coder-32b-instruct:free",
    
    // Medium-large models (14B-30B)
    "mistralai/mistral-small-3.1-24b-instruct:free",
    "mistralai/mistral-small-24b-instruct-2501:free",
    "cognitivecomputations/dolphin3.0-r1-mistral-24b:free",
    "cognitivecomputations/dolphin3.0-mistral-24b:free",
    "google/gemma-3-27b-it:free",
    "google/gemini-2.0-flash-exp:free",
    "rekaai/reka-flash-3:free",
    
    // Medium models (7B-14B)
    "qwen/qwen3-14b:free",
    "deepseek/deepseek-r1-distill-qwen-14b:free",
    "agentica-org/deepcoder-14b-preview:free",
    "moonshotai/moonlight-16b-a3b-instruct:free",
    "opengvlab/internvl3-14b:free",
    "google/gemma-3-12b-it:free",
    "meta-llama/llama-3.2-11b-vision-instruct:free",
    "thudm/glm-4-9b:free",
    "thudm/glm-z1-9b:free",
    "google/gemma-2-9b-it:free",
    "qwen/qwen3-8b:free",
    "meta-llama/llama-3.1-8b-instruct:free",
    "nousresearch/deephermes-3-llama-3-8b-preview:free",
    
    // Specialized models (various sizes)
    "deepseek/deepseek-r1:free",
    "microsoft/phi-4-reasoning-plus:free",
    "microsoft/phi-4-reasoning:free",
    "deepseek/deepseek-v3-base:free",
    "deepseek/deepseek-r1-zero:free",
    "deepseek/deepseek-prover-v2:free",
    "deepseek/deepseek-chat-v3-0324:free",
    "deepseek/deepseek-chat:free",
    "microsoft/mai-ds-r1:free",
    "tngtech/deepseek-r1t-chimera:free",
    "mistralai/mistral-nemo:free",
    
    // Small models (< 7B)
    "qwen/qwen3-4b:free",
    "google/gemma-3-4b-it:free",
    "qwen/qwen-2.5-7b-instruct:free",
    "mistralai/mistral-7b-instruct:free",
    "qwen/qwen-2.5-vl-7b-instruct:free",
    "opengvlab/internvl3-2b:free",
    "google/gemma-3-1b-it:free",
    "meta-llama/llama-3.2-3b-instruct:free",
    "allenai/molmo-7b-d:free",
    "qwen/qwen3-1.7b:free",
    "qwen/qwen2.5-vl-3b-instruct:free",
    "meta-llama/llama-3.2-1b-instruct:free",
    "qwen/qwen3-0.6b-04-28:free",
    
    // Special cases and multimodal models
    "google/learnlm-1.5-pro-experimental:free",
    "moonshotai/kimi-vl-a3b-thinking:free"
];

const MAX_CONSECUTIVE_FAILURES: usize = 3;
const INITIAL_JAIL_HOURS: i64 = 24;

const INITIAL_JAIL_HOURS: i64 = 24;
const JAIL_TIME_MULTIPLIER: i64 = 2;

const JAIL_TIME_MULTIPLIER: i64 = 2;
const MAX_JAIL_HOURS: i64 = 168; // 7 days
const BLACKLIST_AFTER_JAIL_COUNT: usize = 3;

const BLACKLIST_AFTER_JAIL_COUNT: usize = 3;
const BLACKLIST_RETRY_DAYS: i64 = 7;

const BLACKLIST_RETRY_DAYS: i64 = 7;

/// Decides if a model should be used based on its jail/blacklist status
fn is_model_available(model_stats: &Option<&ModelStats>) -> bool {
    match model_stats {
        None => true, // No stats yet, model is available
        Some(stats) => {
            // Check if blacklisted but should be retried
            if stats.blacklisted {
                if let Some(blacklisted_since) = stats.blacklisted_since {
                    let retry_duration = chrono::Duration::days(BLACKLIST_RETRY_DAYS);