use anyhow::Result;
use redis::AsyncCommands;

#[tokio::main]
async fn main() -> Result<()> {
    let client = redis::Client::open(
        "redis://127.0.0.1:6379
",
    )?;

    let mut conn = client.get_multiplexed_async_connection().await?;

    let _: () = conn.set("name", "meng").await?;

    let name: String = conn.get("name").await?;
    println!("name={}", name);

    let _: () = conn.set("counter", 1).await?;
    let value: i32 = conn.get("counter").await?;
    println!("value={}", value);

    let new_value: i32 = conn.incr("counter", 1).await?;
    println!("counter after incr={}", new_value);

    Ok(())
}
