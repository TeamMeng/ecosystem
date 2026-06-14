use adk_rust::{prelude::*, Launcher};
use anyhow::Result;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let model = Arc::new(OllamaModel::new(OllamaConfig::new("qwen2.5:7b"))?);

    // 专家：计费代理
    let billing_agent = LlmAgentBuilder::new("billing_agent")
        .description("处理计费问题：付款、发票、订阅、退款")
        .instruction(
            "您是计费专家。帮助客户处理：\n\
                     - 发票问题和付款历史\n\
                     - 订阅计划和升级\n\
                     - 退款请求\n\
                     - 付款方式更新\n\
                     请保持专业并提供清晰的计费信息。",
        )
        .model(model.clone())
        .build()?;

    // 专家：技术支持代理
    let support_agent = LlmAgentBuilder::new("support_agent")
        .description("处理技术支持：错误、故障、故障排除、操作方法问题")
        .instruction(
            "您是技术支持专家。帮助客户处理：\n\
                     - 故障排除错误和故障\n\
                     - 关于产品使用方法的问题\n\
                     - 配置和设置问题\n\
                     - 性能问题\n\
                     请耐心并提供分步指导。",
        )
        .model(model.clone())
        .build()?;

    // 协调员：路由到适当的专家
    let coordinator = LlmAgentBuilder::new("coordinator")
        .description("主要客户服务协调员")
        .instruction(
            "您是客户服务协调员。分析每个客户请求：\n\n\
                     - 对于计费问题（付款、发票、订阅、退款）：\n\
                       转接给 billing_agent\n\n\
                     - 对于技术问题（错误、故障、操作方法、故障排除）：\n\
                       转接给 support_agent\n\n\
                     - 对于一般问候或不明确的请求：\n\
                       自行回复并提出澄清问题\n\n\
                     转接时，简要地向客户确认并解释移交过程。",
        )
        .model(model.clone())
        .sub_agent(Arc::new(billing_agent))
        .sub_agent(Arc::new(support_agent))
        .build()?;

    println!("🏢 客户服务中心");
    println!("   协调员 → 计费代理 | 支持代理");
    println!();

    Launcher::new(Arc::new(coordinator)).run().await?;
    Ok(())
}
