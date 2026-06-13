use adk_rust::prelude::*;
use adk_rust::Launcher;
use anyhow::Result;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    // 要求：ollama serve && ollama pull llama3.2
    let model = OllamaModel::new(OllamaConfig::new("llama3.2"))?;

    let agent = LlmAgentBuilder::new("assistant")
        .instruction("You are a helpful assistant.")
        .model(Arc::new(model))
        .tool(Arc::new(GoogleSearchTool::new()))
        .build()?;

    Launcher::new(Arc::new(agent)).run().await?;

    Ok(())
}
