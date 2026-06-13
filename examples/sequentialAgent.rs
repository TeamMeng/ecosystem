use adk_rust::{prelude::*, Launcher};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let model = Arc::new(OllamaModel::new(OllamaConfig::new("llama3.2"))?);

    let researcher = LlmAgentBuilder::new("researcher")
        .instruction(
            "Research the given topic. List 3-5 key facts or points. \
                     Be factual and concise.",
        )
        .model(model.clone())
        .output_key("research") // Saves output to state
        .build()?;

    let analyzer = LlmAgentBuilder::new("analyzer")
        .instruction(
            "Based on the research above, identify 2-3 key insights \
                     or patterns. What's the bigger picture?",
        )
        .model(model.clone())
        .output_key("analysis")
        .build()?;

    let summarizer = LlmAgentBuilder::new("summarizer")
        .instruction(
            "Create a brief executive summary combining the research \
                     and analysis. Keep it under 100 words.",
        )
        .model(model.clone())
        .build()?;

    let pipeline = SequentialAgent::new(
        "research_pipeline",
        vec![
            Arc::new(researcher),
            Arc::new(analyzer),
            Arc::new(summarizer),
        ],
    )
    .with_description("Research → Analyze → Summarize");

    println!("📋 Sequential Pipeline: Research → Analyze → Summarize");
    println!();

    Launcher::new(Arc::new(pipeline)).run().await?;

    Ok(())
}
