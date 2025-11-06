fn unjail_all_models(config: &mut SimpleFreeOpenRouterConfig) -> Result<(), String> {
    unjail_model(config, "*")
}