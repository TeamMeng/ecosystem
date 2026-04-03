use std::time::Duration;
use tokio::sync::mpsc;

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Task {
    id: u64,
    payload: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (tx, mut rx) = mpsc::channel::<Task>(100);

    tokio::spawn(async move {
        while let Some(task) = rx.recv().await {
            println!("Processing task: {:?}", task);

            tokio::time::sleep(Duration::from_secs(2)).await;
            println!("Done: {:?}", task);
        }
    });

    for i in 1..=5 {
        let task = Task {
            id: i,
            payload: format!("Task payload {}", i),
        };
        tx.send(task).await?;
    }

    drop(tx);

    tokio::time::sleep(Duration::from_secs(10)).await;

    Ok(())
}
