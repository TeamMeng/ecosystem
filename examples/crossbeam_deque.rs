use crossbeam_deque::{Injector, Steal, Worker};
use std::sync::Arc;

fn work_strealing_concept() {
    let injector = Arc::new(Injector::new());

    let worker1 = Worker::new_fifo();
    let worker2 = Worker::new_lifo();

    let _stealer1 = worker1.stealer();
    let _stealer2 = worker2.stealer();

    for i in 0..10 {
        injector.push(format!("task {}", i));
    }

    for i in 0..5 {
        worker1.push(format!("local task-1 {}", i));
        worker2.push(format!("local task-2 {}", i));
    }

    println!("injector len: {}", injector.len());
    println!("worker1 len: {}", worker1.len());
    println!("worker2 len: {}", worker2.len());
}

fn worker_basic_example() {
    let fifo_worker = Worker::new_fifo();

    fifo_worker.push("task 1");
    fifo_worker.push("task 2");
    fifo_worker.push("task 3");

    println!("fifo len: {}", fifo_worker.len());
    println!("fifo is empty: {}", fifo_worker.is_empty());

    while let Some(task) = fifo_worker.pop() {
        println!("fifo pop: {}", task);
    }

    println!("=====================");

    let lifo_worker = Worker::new_lifo();

    lifo_worker.push("task 1");
    lifo_worker.push("task 2");
    lifo_worker.push("task 3");

    println!("lifo len: {}", lifo_worker.len());
    println!("lifo is empty: {}", lifo_worker.is_empty());

    while let Some(task) = lifo_worker.pop() {
        println!("lifo pop: {}", task);
    }
}

fn worker_stealer_interaction() {
    let worker = Worker::new_fifo();
    let stealer = worker.stealer();

    for i in 0..5 {
        worker.push(format!("task {}", i));
    }

    println!("worker len: {}", worker.len());

    if let Some(task) = worker.pop() {
        println!("work pop {}", task);
    }

    match stealer.steal() {
        Steal::Success(task) => println!("stealer pop {}", task),
        Steal::Empty => println!("stealer is empty"),
        Steal::Retry => println!("stealer retry"),
    }

    println!("worker len: {}", worker.len());

    while let Some(task) = worker.pop() {
        println!("task {}", task)
    }
}

fn main() {
    work_strealing_concept();
    worker_basic_example();
    println!("=====================");
    worker_stealer_interaction();
}
