use tokio::{join, time::Instant};

async fn delayed_task(name: &str, delay_ms: u64) -> String {
    let start = Instant::now();
    println!("task {} start, delay {}ms", name, delay_ms);

    let result = format!(
        "task {} finished, task {:.2}ms",
        name,
        start.elapsed().as_millis()
    );
    println!("{}", result);
    result
}

#[tokio::main]
async fn main() {
    // let start = Instant::now();

    // sleep(Duration::from_secs(1)).await;
    // println!("sleep sec 1: {:.2}", start.elapsed().as_secs_f64());

    // sleep(Duration::from_millis(500)).await;
    // println!("sleep millis 500: {:.2}", start.elapsed().as_secs_f64());

    // sleep(Duration::from_micros(100_00)).await;
    // println!("sleep micros 100: {:.2}", start.elapsed().as_secs_f64());

    // sleep(Duration::from_secs_f64(0.25)).await;
    // println!("sleep sec 0.25: {:.2}", start.elapsed().as_secs_f64());

    let start = Instant::now();

    let (ret1, ret2, ret3) = join!(
        delayed_task("task1", 1000),
        delayed_task("task2", 500),
        delayed_task("task3", 250)
    );

    println!("{}", ret1);
    println!("{}", ret2);
    println!("{}", ret3);

    println!("total take: {:.2}", start.elapsed().as_millis());
}
