use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    println!("Hello, Tokio!");

    // wait 1 secs
    sleep(Duration::from_secs(1)).await;

    println!("1 secs");

    let task1 = async {
        for i in 1..=3 {
            println!("task1: step {}", i);
            sleep(Duration::from_millis(500)).await;
        }
    };

    let task2 = async {
        for i in 1..=3 {
            println!("task2: step {}", i);
            sleep(Duration::from_millis(500)).await;
        }
    };

    let task3 = async {
        for i in 1..=3 {
            println!("task3: step {}", i);
            sleep(Duration::from_millis(500)).await;
        }
    };

    tokio::join!(task1, task2, task3);
    println!("all task finish");
}
