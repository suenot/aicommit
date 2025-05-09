#!/usr/bin/env python3

import json
import os
import requests
import sys
from pathlib import Path

def load_config():
    """Load aicommit configuration file."""
    config_path = Path.home() / ".aicommit.json"
    
    if not config_path.exists():
        print(f"Config file not found at {config_path}")
        sys.exit(1)
    
    with open(config_path, 'r') as f:
        return json.load(f)

def get_openrouter_api_key():
    """Extract OpenRouter API key from aicommit config."""
    config = load_config()
    
    for provider in config.get('providers', []):
        # Check all provider types that might have an OpenRouter API key
        if 'OpenRouter' in str(provider):
            provider_data = list(provider.values())[0]  # Extract the provider data
            if 'api_key' in provider_data:
                return provider_data['api_key']
        elif isinstance(provider, dict) and provider.get('provider') in ['openrouter', 'simple_free_openrouter']:
            if 'api_key' in provider:
                return provider['api_key']
    
    print("No OpenRouter API key found in config")
    sys.exit(1)

def get_free_models(api_key):
    """Fetch all models from OpenRouter API and filter for free ones."""
    headers = {
        "Authorization": f"Bearer {api_key}",
        "HTTP-Referer": "https://suenot.github.io/aicommit/",
        "X-Title": "aicommit"
    }
    
    response = requests.get("https://openrouter.ai/api/v1/models", headers=headers)
    
    if not response.ok:
        print(f"API request failed: {response.status_code} - {response.text}")
        sys.exit(1)
    
    models_data = response.json()
    
    free_models = []
    all_models = []
    
    for model in models_data.get('data', []):
        model_id = model.get('id')
        if not model_id:
            continue
        
        # Extract parameters size from model ID if available
        model_size = "Unknown"
        for size_pattern in ["253b", "235b", "200b", "124b", "70b", "80b", "90b", "72b", "65b", 
                            "40b", "32b", "30b", "24b", "20b", "16b", "14b", "13b", "12b", "11b", 
                            "10b", "9b", "8b", "7b", "6b", "5b", "4b", "3b", "2b", "1b"]:
            if size_pattern.lower() in model_id.lower():
                model_size = size_pattern.upper()
                break
            
        # Add to all models
        all_models.append({
            'id': model_id,
            'name': model.get('name', ''),
            'size': model_size,
            'context_length': model.get('context_length', 0),
            'pricing': {
                'prompt': model.get('pricing', {}).get('prompt', 0),
                'completion': model.get('pricing', {}).get('completion', 0)
            },
            'free': model.get('free', False),
            'free_tokens': model.get('free_tokens', 0)
        })
        
        # Check if model is free (multiple indicators)
        is_free = False
        
        # 1. Look for ":free" in the model ID
        if ":free" in model_id:
            is_free = True
        # 2. Check for free property
        elif model.get('free', False):
            is_free = True
        # 3. Check for free_tokens
        elif model.get('free_tokens', 0) > 0:
            is_free = True
        # 4. Check if pricing is zero
        elif model.get('pricing', {}).get('prompt', 1) == 0 and model.get('pricing', {}).get('completion', 1) == 0:
            is_free = True
            
        if is_free:
            free_models.append({
                'id': model_id,
                'name': model.get('name', ''),
                'size': model_size,
                'context_length': model.get('context_length', 0),
                'free_tokens': model.get('free_tokens', 0)
            })
    
    return free_models, all_models

def main():
    try:
        api_key = get_openrouter_api_key()
        free_models, all_models = get_free_models(api_key)
        
        # Save results to files
        output_dir = Path('openrouter_models')
        output_dir.mkdir(exist_ok=True)
        
        # Save free models
        with open(output_dir / 'free_models.json', 'w') as f:
            json.dump(free_models, f, indent=2)
        
        # Save all models
        with open(output_dir / 'all_models.json', 'w') as f:
            json.dump(all_models, f, indent=2)
        
        # Create a readable text file with free models
        with open(output_dir / 'free_models.txt', 'w') as f:
            f.write("OpenRouter Free Models:\n\n")
            # Sort by size (parameter count) if available
            sorted_models = sorted(free_models, 
                                  key=lambda x: (0 if x.get('size', 'Unknown') == 'Unknown' else 
                                                int(x.get('size', '0B').lower().replace('b', ''))), 
                                  reverse=True)
            
            for model in sorted_models:
                f.write(f"ID: {model.get('id')}\n")
                f.write(f"Name: {model.get('name', 'Unknown')}\n")
                f.write(f"Size: {model.get('size', 'Unknown')}\n")
                f.write(f"Context Length: {model.get('context_length')}\n")
                f.write(f"Free Tokens: {model.get('free_tokens', 0)}\n")
                f.write("-" * 50 + "\n\n")
        
        # Print results
        print(f"Found {len(free_models)} free models out of {len(all_models)} total models")
        print(f"Results saved to {output_dir} directory")
        print("\nTop 10 Free Models (by size):")
        
        # Sort by parameter size for display
        for i, model in enumerate(sorted(free_models, 
                                          key=lambda x: (0 if x.get('size', 'Unknown') == 'Unknown' else 
                                                       int(x.get('size', '0B').lower().replace('b', ''))), 
                                          reverse=True)[:10]):
            print(f"{i+1}. {model['id']} ({model.get('size', 'Unknown')})")
            
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main() 