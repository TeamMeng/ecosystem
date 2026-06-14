use adk_rust::{prelude::*, Launcher};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let model = Arc::new(OllamaModel::new(OllamaConfig::new("llama3.2"))?);

    let technical = LlmAgentBuilder::new("technical_analyst")
        .instruction(
            "You are a senior software architect. \
                     FOCUS ONLY ON: code quality, system architecture, scalability, \
                     security vulnerabilities, and tech stack choices. \
                     Start your response with '🔧 TECHNICAL:' and give 2-3 bullet points.",
        )
        .model(model.clone())
        .build()?;

    let business = LlmAgentBuilder::new("business_analyst")
        .instruction(
            "You are a business strategist and MBA graduate. \
                     FOCUS ONLY ON: market opportunity, revenue model, competition, \
                     cost structure, and go-to-market strategy. \
                     Start your response with '💼 BUSINESS:' and give 2-3 bullet points.",
        )
        .model(model.clone())
        .build()?;

    let user_exp = LlmAgentBuilder::new("ux_analyst")
        .instruction(
            "You are a UX researcher and designer. \
                     FOCUS ONLY ON: user journey, accessibility, pain points, \
                     visual design, and user satisfaction metrics. \
                     Start your response with '🎨 UX:' and give 2-3 bullet points.",
        )
        .model(model.clone())
        .build()?;

    let multi_analyst = ParallelAgent::new(
        "multi_perspertive",
        vec![Arc::new(technical), Arc::new(business), Arc::new(user_exp)],
    )
    .with_description("Technical + Business + UX analysis in parallel");

    println!("⚡ Parallel Analysis: Technical | Business | UX");
    println!("   (All three run simultaneously!)");
    println!();

    Launcher::new(Arc::new(multi_analyst)).run().await?;

    Ok(())
}
