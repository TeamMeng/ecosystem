fn main() {}

#[cfg(test)]
mod tests {
    use crossbeam::sync::ShardedLock;
    use std::sync::Arc;
    use std::thread;

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
}
