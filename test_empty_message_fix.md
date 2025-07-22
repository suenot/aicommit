# Test for Empty Commit Message Fix

## Problem Description
The aicommit tool was generating empty commit messages, causing git to abort with:
```
Error: "Aborting commit due to empty commit message.\n"
```

## Root Cause
The `generate_simple_free_commit_message` function (and other provider implementations) were not validating or cleaning the API response content, allowing empty or whitespace-only messages to be passed to git commit.

## Fix Applied
Added message validation and cleaning to all provider implementations:

1. **SimpleFreeOpenRouter** - Added validation in `generate_simple_free_commit_message`
2. **OpenRouter** - Added validation in `generate_openrouter_commit_message`  
3. **OpenAICompatible** - Added validation in `generate_openai_compatible_commit_message`
4. **Ollama** - Already had validation (was working correctly)

## Validation Logic
```rust
// Clean and validate the message
let message = raw_message
    .trim()
    .trim_start_matches(['\\', '/', '-', ' '])
    .trim_end_matches(['\\', '/', '-', ' ', '.'])
    .trim()
    .to_string();

if message.is_empty() || message.len() < 3 {
    return Err("Generated commit message is too short or empty".to_string());
}
```

## Additional Safety Measures
- Added final validation in main commit flow before calling `create_git_commit`
- Added final validation in dry-run mode before returning message
- Consistent error handling across all provider implementations

## Expected Behavior After Fix
- Empty or whitespace-only API responses will be caught and reported as errors
- Users will see clear error messages instead of git aborting
- The tool will retry with different models (for SimpleFreeOpenRouter) when messages are invalid
- Consistent behavior across all provider types
