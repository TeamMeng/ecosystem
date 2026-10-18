use anyhow::{bail, Context, Result};
use config::{Config, Environment, File};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Deserialize, Clone)]
struct ChatReply {
    content: String,
    prompt_tokens: u32,
    completion_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
struct Choice {
    message: ChatMessage,
    finish_reason: String,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    id: String,
    created: i64,
    model: String,
    usage: Usage,
    choices: Vec<Choice>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Provider {
    Deepseek,
}

#[derive(Debug, Deserialize)]
struct AppSettings {
    provider: Provider,
    model: String,
    api_key: String,
    #[serde(default)]
    base_url: Option<String>,
}

fn load_settings() -> Result<AppSettings> {
    dotenvy::dotenv().ok();

    let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "dev".to_string());
    let raw_config = Config::builder()
        .set_default("provider", "deepseek")?
        .set_default("model", "deepseek-flash")?
        .add_source(File::with_name(&format!("config/{run_mode}")).required(false))
        .add_source(Environment::with_prefix("APP").separator("__"))
        .build()
        .context("Failed to build layered configuration")?;

    raw_config
        .try_deserialize()
        .context("Failed to deserialize configuration into AppSettings")
}

fn build_client(settings: &AppSettings) -> Box<dyn LlmClient> {
    match settings.provider {
        Provider::Deepseek => Box::new(DeepseekClient::new(
            settings
                .base_url
                .clone()
                .unwrap_or_else(|| "https://api.deepseek.com".to_string()),
            settings.api_key.clone(),
            settings.model.clone(),
        )),
    }
}

#[async_trait::async_trait]
trait LlmClient: Send + Sync {
    async fn chat(&self, system_prompt: &str, user_prompt: &str) -> Result<ChatReply>;
}

async fn report(name: &str, client: &dyn LlmClient) {
    let system = "You are a concise Rust teaching assistant.";
    let prompt = "Explain Rust ownership in one sentence.";

    match client.chat(system, prompt).await {
        Ok(replt) => println!(
            "[{name}] {} (tokens: {} + {})\n",
            replt.content, replt.prompt_tokens, replt.completion_tokens
        ),
        Err(err) => println!("[{name}] error: {err:#?}\n"),
    }
}

#[allow(unused)]
struct DeepseekClient {
    http: Client,
    base_url: String,
    api_key: String,
    model: String,
}

impl DeepseekClient {
    fn new(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            http: Client::new(),
            base_url: base_url.into(),
            api_key: api_key.into(),
            model: model.into(),
        }
    }
}

#[async_trait::async_trait]
impl LlmClient for DeepseekClient {
    async fn chat(&self, system_prompt: &str, user_prompt: &str) -> Result<ChatReply> {
        let endpoint = format!("{}/chat/completions", self.base_url);

        let request_body = ChatCompletionRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_prompt.to_string(),
                },
            ],
        };

        let response = self
            .http
            .post(&endpoint)
            .bearer_auth(&self.api_key)
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            bail!("status code {} with body {}", status, body)
        }

        let parsed: ChatCompletionResponse = serde_json::from_str(&body)?;

        let reply = parsed
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .unwrap_or_default();

        Ok(ChatReply {
            content: reply,
            prompt_tokens: parsed.usage.prompt_tokens,
            completion_tokens: parsed.usage.completion_tokens,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let settings = load_settings()?;
    let ds = build_client(&settings);

    let clients: Vec<(&str, Box<dyn LlmClient>)> = vec![("deepseek-flash", ds)];

    for (name, client) in clients.iter() {
        report(name, client.as_ref()).await;
    }

    Ok(())
}
