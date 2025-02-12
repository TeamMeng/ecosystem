use anyhow::Result;
use rustis::{
    client::Client,
    commands::{GenericCommands, StringCommands},
};

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::connect("127.0.0.1:6379").await?;

    client.set("key", "value").await?;

    let value: String = client.get("key").await?;
    println!("value: {}", value);

    let v = client.del("key").await?;
    println!("v: {}", v);

    let value: String = client.get("key").await?;
    println!("value: {}", value);

    client.set("key", 200).await?;
    client.decr("key").await?;

    let value: String = client.get("key").await?;
    println!("value: {}", value);

    Ok(())
}
