use anyhow::{Context, Result};
use config::{Config, Environment, File};
use serde::Deserialize;
use std::env;

#[allow(unused)]
#[derive(Debug, Deserialize)]
struct Settings {
    api_key: String,
    model: String,
    max_tokens: u32,
    request_timeout_sces: u64,
}

fn load_config() -> Result<Settings> {
    dotenvy::dotenv().ok();

    let run_demo = env::var("RUN_MODE").unwrap_or("dev".to_string());
    let raw_config = Config::builder()
        .set_default("model", "deepseek-v4.1-flash")?
        .set_default("max_tokens", 1024)?
        .set_default("request_timeout_sces", 30)?
        .add_source(File::with_name(&format!("config/{}", run_demo)).required(false))
        .add_source(Environment::with_prefix("APP").separator("__"))
        .build()
        .context("failed to build configuration")?;

    raw_config
        .try_deserialize()
        .context("failed to deserialize configuration")
}

fn main() -> Result<()> {
    let settings = load_config()?;
    println!("{:#?}", settings);

    Ok(())
}
