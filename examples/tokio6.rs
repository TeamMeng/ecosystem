use anyhow::Result;
use std::time::Instant;
use tokio::{
    runtime::Handle,
    time::{interval, sleep, Duration},
};

#[tokio::main]
async fn main() -> Result<()> {
    let monitor_task = tokio::spawn(async move {
        let mut interavl = interval(Duration::from_secs(1));

        loop {
            interavl.tick().await;

            let _handle = Handle::current();
            println!("Runtime - time: {:?}", Instant::now());
        }
    });

    let work_tasks = (1..=3)
        .map(|i| {
            tokio::spawn(async move {
                for j in 1..=5 {
                    println!("work task {} - step {}", i, j);
                    sleep(Duration::from_millis(300)).await;
                }
            })
        })
        .collect::<Vec<_>>();

    for task in work_tasks {
        task.await.unwrap();
    }

    monitor_task.abort();

    Ok(())
}
