use crossbeam::{
    atomic::AtomicCell,
    sync::{Parker, ShardedLock},
};
use crossbeam_channel::unbounded;
use std::{
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

fn atomic_cell_basic() {
    let cell = AtomicCell::new(Point { x: 0.0, y: 0.0 });

    let current = cell.load();
    println!("Current Point: ({}, {})", current.x, current.y);

    cell.store(Point { x: 1.0, y: 2.0 });
    println!("Updated Point: ({}, {})", cell.load().x, cell.load().y);

    let old = cell.swap(Point { x: 3.0, y: 4.0 });
    println!("Old Point: ({}, {})", old.x, old.y);

    println!(
        "Current Point after swap: ({}, {})",
        cell.load().x,
        cell.load().y
    );
}

fn atomic_cell_concurrent() {
    let cell = Arc::new(AtomicCell::new(0u64));
    let mut handlers = vec![];

    for i in 0..10 {
        let cell = Arc::clone(&cell);
        let handle = std::thread::spawn(move || {
            for j in 0..1000 {
                let current = cell.load();
                cell.store(current + 1);

                if j % 100 == 0 {
                    println!(
                        "thread {}: operation {}, Current value: {}",
                        i,
                        j,
                        cell.load()
                    );
                }
            }
        });
        handlers.push(handle);
    }

    for handler in handlers {
        handler.join().unwrap();
    }

    println!("Final value: {}", cell.load());
}

fn parker_basic() {
    let parker = Parker::new();
    let unparker = parker.unparker().clone();

    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(1));
        println!("waker: unparking the thread");
        unparker.unpark();
    });

    println!("parker: parking the thread");
    parker.park();
    println!("parker: thread unparked, resuming execution");
}

fn parker_timeout() {
    let parker = Parker::new();

    let start = Instant::now();
    println!("parker: parking the thread with a timeout of 2 seconds");

    parker.park_timeout(Duration::from_secs(2));
    println!(
        "parker: thread unparked or timeout elapsed, resuming execution after {:?}",
        start.elapsed()
    );
}

fn parker_cross_thread() {
    let (unparker_tx, unparker_rx) = unbounded();

    let worker = thread::spawn(move || {
        let worker_parker = Parker::new();
        let worker_unparker = worker_parker.unparker().clone();

        unparker_tx.send(worker_unparker).unwrap();

        println!("worker: waiting for unparker");
        worker_parker.park();
        println!("worker wake up");

        for i in 0..3 {
            println!("working task: {}", i);
            thread::sleep(Duration::from_millis(200));
        }

        println!("worker finish");
    });

    let worker_unparker = unparker_rx.recv().unwrap();
    thread::sleep(Duration::from_secs(2));

    worker_unparker.unpark();
    worker.join().unwrap();
}

fn shared_lock_performance() {
    let data = Arc::new(ShardedLock::new(vec![0; 1000]));
    let mut handlers = vec![];

    let start = Instant::now();

    for i in 0..10 {
        let data = Arc::clone(&data);
        let handle = thread::spawn(move || {
            for j in 0..10000 {
                let guard = data.read().unwrap();
                let _sum: i32 = guard.iter().sum();

                if j % 1000 == 0 {
                    println!("thread {}: read operation {}", i, j,);
                }
            }
        });
        handlers.push(handle);
    }

    let data_writer = Arc::clone(&data);
    let writer_handle = thread::spawn(move || {
        for i in 0..100 {
            thread::sleep(Duration::from_millis(10));
            {
                let mut guard = data_writer.write().unwrap();
                let len = guard.len();
                guard[i % len] += 1;
            }

            if i % 10 == 0 {
                println!("Writer: updated data at iteration {}", i);
            }
        }
    });
    handlers.push(writer_handle);

    for handle in handlers {
        handle.join().unwrap();
    }

    let elapsed = start.elapsed();
    println!("Total execution time: {:.2?}", elapsed);

    let guard = data.read().unwrap();
    println!("Final data: {:?}", guard.iter().sum::<i32>());
}

fn main() {
    atomic_cell_basic();
    atomic_cell_concurrent();
    parker_basic();
    parker_timeout();
    parker_cross_thread();
    shared_lock_performance();
}

#[cfg(test)]
mod tests {
    use crossbeam::atomic::AtomicCell;
    use crossbeam::sync::ShardedLock;
    use crossbeam::thread;
    use crossbeam_channel::bounded;
    use crossbeam_utils::CachePadded;
    use std::{
        sync::{
            atomic::{AtomicU64, Ordering},
            Arc,
        },
        thread,
        thread::sleep,
        time::Duration,
        time::Instant,
    };

    struct Counter {
        counter1: CachePadded<AtomicU64>,
        counter2: CachePadded<AtomicU64>,
    }

    fn fibonacci(n: u64) -> u64 {
        match n {
            0 => 0,
            1 => 1,
            _ => fibonacci(n - 1) + fibonacci(n - 2),
        }
    }

    #[test]
    fn simple() {
        let data = Arc::new(ShardedLock::new(vec![1, 2, 3, 4, 5]));
        let mut handlers = vec![];

        for i in 0..4 {
            let data = Arc::clone(&data);
            let handle = thread::spawn(move || {
                let guard = data.read().unwrap();
                println!("thread {} read data: {:?}", i, *guard);
            });
            handlers.push(handle);
        }

        for handle in handlers {
            handle.join().unwrap();
        }
    }

    #[test]
    fn scope() {
        let data = vec![1, 2, 3, 4, 5];

        crossbeam_utils::thread::scope(|s| {
            let _handler1 = s.spawn(|_| {
                let sum: i32 = data[0..2].iter().sum();
                println!("Sum: {}", sum);
            });

            let _handler2 = s.spawn(|_| {
                let sum: i32 = data[3..5].iter().sum();
                println!("Sum: {}", sum);
            });
        })
        .unwrap();

        println!("{:?}", data);
    }

    #[test]
    fn atomic_test() {
        let counter = Arc::new(AtomicCell::new(0u64));
        let mut handlers = vec![];
        for _ in 0..10 {
            let counter = Arc::clone(&counter);
            let handle = std::thread::spawn(move || {
                for _ in 0..1000 {
                    let old = counter.load();
                    counter.store(old + 1);
                }
            });
            handlers.push(handle);
        }

        for handle in handlers {
            handle.join().unwrap();
        }
        println!("Final counter value: {}", counter.load());
    }

    #[test]
    fn counter_test() {
        let counters = Arc::new(Counter {
            counter1: CachePadded::new(AtomicU64::new(0)),
            counter2: CachePadded::new(AtomicU64::new(0)),
        });

        let mut handlers = vec![];

        let counters1 = Arc::clone(&counters);
        let handle1 = std::thread::spawn(move || {
            for _ in 0..1_000_000 {
                counters1.counter2.fetch_add(1, Ordering::Relaxed);
            }
        });
        handlers.push(handle1);

        let counters2 = Arc::clone(&counters);
        let handle2 = std::thread::spawn(move || {
            for _ in 0..1_000_000 {
                counters2.counter1.fetch_add(1, Ordering::Relaxed);
            }
        });
        handlers.push(handle2);

        for handle in handlers {
            handle.join().unwrap();
        }

        println!(
            "Counter 1: {}, Counter 2: {}",
            counters.counter1.load(Ordering::Relaxed),
            counters.counter2.load(Ordering::Relaxed)
        );
    }

    #[test]
    fn channel_test() {
        let numbers = vec![10, 20, 30, 40, 50];
        let start_time = Instant::now();

        let (task_sender, task_receiver) = bounded(numbers.len());
        let (result_sender, result_receiver) = bounded(numbers.len());

        for num in numbers.iter() {
            task_sender.send(num).unwrap();
        }
        drop(task_sender);

        thread::scope(|s| {
            for word_id in 0..4 {
                let task_receiver = task_receiver.clone();
                let result_sender = result_sender.clone();
                s.spawn(move |_| {
                    while let Ok(num) = task_receiver.recv() {
                        let result = fibonacci(*num);
                        result_sender.send((num, result)).unwrap();
                        println!(
                            "Worker {} computed Fibonacci({}) = {}",
                            word_id, num, result
                        );
                    }
                });
            }

            drop(result_sender);

            let mut results = Vec::new();
            while let Ok((num, result)) = result_receiver.recv() {
                results.push((num, result));
            }

            results.sort_by_key(|&(num, _)| num);

            for (num, result) in results {
                println!("Fibonacci({}) = {}", num, result);
            }
        })
        .unwrap();

        let elapsed = start_time.elapsed();
        println!("Total execution time: {:.2?}", elapsed);
    }

    #[test]
    fn producer_consumer_example() {
        let (tx, rx) = bounded(10);

        thread::scope(|s| {
            s.spawn(|_| {
                for i in 0..20 {
                    println!("producer: {}", i);
                    tx.send(i).unwrap();
                    sleep(Duration::from_millis(100));
                }
            });

            s.spawn(|_| {
                for _ in 0..20 {
                    let value = rx.recv().unwrap();
                    println!("consumer: {}", value);
                    sleep(Duration::from_millis(150));
                }
            });
        })
        .unwrap();
    }
}
