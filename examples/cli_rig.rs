use rig::client::CompletionClient;
use rig::client::Nothing;
use rig::http_client::ReqwestClient;
use rig::integrations::cli_chatbot;
use rig::providers::ollama;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 显式禁用系统代理探测，避免某些 macOS 环境下 reqwest 初始化时触发 panic。
    let http_client = ReqwestClient::builder().no_proxy().build()?;
    let client = ollama::Client::builder()
        .http_client(http_client)
        .base_url("http://127.0.0.1:11434")
        .api_key(Nothing)
        .build()?;

    // 创建 agent（指定模型）
    let agent = client
        .agent("llama3.2") // 你本地 ollama 的模型名
        .preamble("You are a helpful assistant.")
        .build();

    cli_chatbot::ChatBotBuilder::new()
        .agent(agent)
        .build()
        .run()
        .await?;

    Ok(())
}
