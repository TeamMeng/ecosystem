use adk_rust::{prelude::*, Launcher};
use anyhow::Result;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let model = Arc::new(OllamaModel::new(OllamaConfig::new("qwen2.5:7b"))?);

    // Create specialist agents
    let tech_agent: Arc<dyn Agent> = Arc::new(
        LlmAgentBuilder::new("tech_expert")
            .instruction("You are a senior software engineer. Be precise and technical.")
            .model(model.clone())
            .build()?,
    );

    let general_agent: Arc<dyn Agent> = Arc::new(
        LlmAgentBuilder::new("general_helper")
            .instruction("You are a friendly assistant. Explain simply, use analogies.")
            .model(model.clone())
            .build()?,
    );

    let creative_agent: Arc<dyn Agent> = Arc::new(
        LlmAgentBuilder::new("creative_writer")
            .instruction("You are a creative writer. Be imaginative and expressive.")
            .model(model.clone())
            .build()?,
    );

    // LLM classifies the query and routes accordingly
    let router = LlmConditionalAgentBuilder::new("smart_router", model.clone())
        .instruction(
            "Classify the user's question as exactly ONE of: \
                     'technical' (coding, debugging, architecture), \
                     'general' (facts, knowledge, how-to), \
                     'creative' (writing, stories, brainstorming). \
                     Respond with ONLY the category name.",
        )
        .route("technical", tech_agent)
        .route("general", general_agent.clone())
        .route("creative", creative_agent)
        .default_route(general_agent)
        .build()?;

    println!("🧠 基于 LLM 的智能路由器");
    Launcher::new(Arc::new(router)).run().await?;
    Ok(())
}
