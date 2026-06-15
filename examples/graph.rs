use adk_graph::{
    edge::{END, START},
    state::State,
    AgentNode, ExecutionConfig, GraphAgent, NodeOutput,
};
use adk_rust::prelude::*;
use anyhow::Result;
use serde_json::json;
use std::{collections::HashMap, sync::Arc};

#[tokio::main]
async fn main() -> Result<()> {
    let model = Arc::new(OllamaModel::new(OllamaConfig::new("qwen2.5:7b"))?);

    // Create specialized LLM agents
    let translator_agent = Arc::new(
        LlmAgentBuilder::new("translator")
            .description("Translates text to French")
            .model(model.clone())
            .instruction("Translate the input text to French. Only output the translation.")
            .build()?,
    );

    let summarizer_agent = Arc::new(
        LlmAgentBuilder::new("summarizer")
            .description("Summarizes text")
            .model(model.clone())
            .instruction("Summarize the input text in one sentence.")
            .build()?,
    );

    let translator_node = AgentNode::new(translator_agent)
        .with_input_mapper(|state| {
            let text = state.get("input").and_then(|v| v.as_str()).unwrap_or("");
            adk_core::Content::new("user").with_text(text)
        })
        .with_output_mapper(|events| {
            let mut updates = HashMap::new();
            for event in events {
                if let Some(content) = event.content() {
                    let text: String = content
                        .parts
                        .iter()
                        .flat_map(|p| p.text())
                        .collect::<Vec<_>>()
                        .join("");
                    if !text.is_empty() {
                        updates.insert("translation".to_string(), json!(text));
                    }
                }
            }
            updates
        });

    let summarizer_node = AgentNode::new(summarizer_agent)
        .with_input_mapper(|state| {
            let text = state.get("input").and_then(|v| v.as_str()).unwrap_or("");
            adk_core::Content::new("user").with_text(text)
        })
        .with_output_mapper(|events| {
            let mut updates = std::collections::HashMap::new();
            for event in events {
                if let Some(content) = event.content() {
                    let text: String = content
                        .parts
                        .iter()
                        .filter_map(|p| p.text())
                        .collect::<Vec<_>>()
                        .join("");
                    if !text.is_empty() {
                        updates.insert("summary".to_string(), json!(text));
                    }
                }
            }
            updates
        });

    let agent = GraphAgent::builder("text_processor")
        .description("Processes text with translation and summarization in parallel")
        .channels(&["input", "translation", "summary", "result"])
        .node(translator_node)
        .node(summarizer_node)
        .node_fn("combine", |ctx| async move {
            let translation = ctx
                .get("translation")
                .and_then(|v| v.as_str())
                .unwrap_or("N/A");
            let summary = ctx.get("summary").and_then(|v| v.as_str()).unwrap_or("N/A");

            let result = format!(
                "=== Processing Complete ===\n\n\
                French Translation:\n{}\n\n\
                Summary:\n{}",
                translation, summary
            );

            Ok(NodeOutput::new().with_update("result", json!(result)))
        })
        // Parallel execution: both nodes start simultaneously
        .edge(START, "translator")
        .edge(START, "summarizer")
        .edge("translator", "combine")
        .edge("summarizer", "combine")
        .edge("combine", END)
        .build()?;

    // Execute the graph
    let mut input = { State::new() };
    input.insert(
        "input".to_string(),
        json!("AI is transforming how we work and live."),
    );

    let result = agent
        .invoke(input, ExecutionConfig::new("thread-1"))
        .await?;
    println!(
        "{}",
        result.get("result").and_then(|v| v.as_str()).unwrap_or("")
    );

    Ok(())
}
