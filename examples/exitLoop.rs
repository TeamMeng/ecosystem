use adk_rust::{prelude::*, Launcher};
use anyhow::Result;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let model = Arc::new(OllamaModel::new(OllamaConfig::new("qwen2.5:7b"))?);

    let generator = LlmAgentBuilder::new("generator")
        .instruction(
            "Generate the initial content requested by the user. \
             Output only the content, not explanations.",
        )
        .model(model.clone())
        .build()?;

    let critic = LlmAgentBuilder::new("critic")
        .instruction(
            "Review ONLY the latest tagline, not the user's original prompt. \
                Score it from 1 to 10. \
                If the tagline is already good enough, give score 8 or higher. \
                Then list concise improvements.",
        )
        .model(model.clone())
        .build()?;

    let refiner = LlmAgentBuilder::new("refiner")
        .instruction(
            "Improve the latest generated content based on the critic's feedback. \
                Output ONLY the improved content. \
                Do not include explanations. \
                If the critic's score is 8 or higher, you MUST call the exit_loop tool.",
        )
        .model(model.clone())
        .tool(Arc::new(ExitLoopTool::new()))
        .build()?;

    let first_draft = SequentialAgent::new("first_draft", vec![Arc::new(generator)]);

    let critique_refine = SequentialAgent::new(
        "critique_refine_step",
        vec![Arc::new(critic), Arc::new(refiner)],
    );

    let iterative_improver = SequentialAgent::new(
        "iterative_improver",
        vec![
            Arc::new(first_draft),
            Arc::new(
                LoopAgent::new("loop", vec![Arc::new(critique_refine)])
                    .with_max_iterations(3)
                    .with_description("Critique-refine loop"),
            ),
        ],
    );

    println!("🔄 Iterative Improvement Loop");
    println!("   generator → critic → refiner → repeat");
    println!();

    Launcher::new(Arc::new(iterative_improver)).run().await?;

    Ok(())
}
