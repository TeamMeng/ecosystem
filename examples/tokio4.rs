use std::{
    thread,
    time::{Duration, Instant},
};

fn sync_example() {
    println!("start task 1");
    thread::sleep(Duration::from_secs(2));
    println!("finish task 1");

    println!("start task 2");
    thread::sleep(Duration::from_secs(2));
    println!("finish task 2");

    println!("all task take 4 sec");
}

#[tokio::main]
async fn main() {
    sync_example();
    println!("----------------------");

    let start = Instant::now();

    let task1 = async {
        println!("start task 1");
        thread::sleep(Duration::from_secs(2));
        println!("finish task 1");
    };

    let task2 = async {
        println!("start task 2");
        thread::sleep(Duration::from_secs(2));
        println!("finish task 2");
    };

    tokio::join!(task1, task2);

    println!("all task finish take {}", start.elapsed().as_secs_f64());
}
