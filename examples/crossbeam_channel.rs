use crossbeam::select;
use crossbeam_channel::{
    after, bounded, never, tick, unbounded, RecvTimeoutError, SendTimeoutError, TryRecvError,
    TrySendError,
};
use std::{
    thread,
    time::{Duration, Instant},
};

fn bounded_channel_example() {
    let (sender, receiver) = bounded(1);

    let sender1 = sender.clone();
    let sender2 = sender.clone();

    let producer1 = thread::spawn(move || {
        for i in 0..5 {
            match sender1.send(i) {
                Ok(()) => println!("Sent {}", i),
                Err(e) => eprintln!("Failed to send {}", e),
            }
        }
        thread::sleep(Duration::from_millis(100));
    });

    let producer2 = thread::spawn(move || {
        for i in 100..105 {
            match sender2.send(i) {
                Ok(()) => println!("Sent {}", i),
                Err(e) => eprintln!("Failed to send {}", e),
            }
        }
        thread::sleep(Duration::from_millis(100));
    });

    let consumer = thread::spawn(move || {
        for received in receiver {
            println!("Received {}", received);
        }
    });

    producer1.join().unwrap();
    producer2.join().unwrap();

    drop(sender);

    consumer.join().unwrap();
}

fn unbounded_channel_example() {
    let (sender, receiver) = unbounded();

    let produer = thread::spawn(move || {
        for i in 1..=1000 {
            sender.send(format!("message {}", i)).unwrap();
            if i % 100 == 0 {
                println!("Send {} messages", i);
            }
        }
        println!("All messages sent");
    });

    let consumer = thread::spawn(move || {
        let mut counter = 0;
        for message in receiver {
            counter += 1;
            if counter % 100 == 0 {
                println!("Received {} messages: {}", counter, message);
                thread::sleep(Duration::from_millis(10));
            }
        }
        println!("Total messages received: {}", counter);
    });

    produer.join().unwrap();
    consumer.join().unwrap();
}

fn zero_capacity_channel_example() {
    let (sender, receiver) = bounded(0);

    let producer = thread::spawn(move || {
        for i in 1..=5 {
            println!("Producer: Sending {}", i);
            sender.send(i).unwrap();
            println!("Producer: Sent {}", i);
        }
    });

    let consumer = thread::spawn(move || {
        for _ in 1..=5 {
            thread::sleep(Duration::from_millis(500));
            println!("Consumer: Waiting to receive...");
            let received = receiver.recv().unwrap();
            println!("Consumer: Received {}", received);
        }
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}

fn non_blocking_operations() {
    let (sender, receiver) = bounded(2);

    match sender.try_send(1) {
        Ok(_) => println!("Sent successfully 1"),
        Err(TrySendError::Full(val)) => println!("Channel is full, failed to send {}", val),
        Err(TrySendError::Disconnected(val)) => {
            println!("Channel is disconnected, failed to send {}", val)
        }
    }

    sender.try_send(2).unwrap();

    match sender.try_send(3) {
        Ok(_) => println!("Sent successfully 3"),
        Err(TrySendError::Full(val)) => println!("Channel is full, failed to send {}", val),
        Err(TrySendError::Disconnected(val)) => {
            println!("Channel is disconnected, failed to send {}", val)
        }
    }

    match receiver.try_recv() {
        Ok(val) => println!("Received successfully {}", val),
        Err(TryRecvError::Empty) => println!("Channel is empty, failed to receive"),
        Err(TryRecvError::Disconnected) => println!("Channel is disconnected, failed to receive"),
    }

    match receiver.try_recv() {
        Ok(val) => println!("Received successfully {}", val),
        Err(TryRecvError::Empty) => println!("Channel is empty, failed to receive"),
        Err(TryRecvError::Disconnected) => println!("Channel is disconnected, failed to receive"),
    }

    match receiver.try_recv() {
        Ok(val) => println!("Received successfully {}", val),
        Err(TryRecvError::Empty) => println!("Channel is empty, failed to receive"),
        Err(TryRecvError::Disconnected) => println!("Channel is disconnected, failed to receive"),
    }
}

fn timeout_operations() {
    let (sender, receiver) = bounded(1);

    sender.send(1).unwrap();

    match sender.send_timeout(2, Duration::from_millis(1000)) {
        Ok(()) => println!("Sent successfully 2"),
        Err(SendTimeoutError::Timeout(val)) => println!("Timeout, failed to send {}", val),
        Err(SendTimeoutError::Disconnected(val)) => {
            println!("Channel is disconnected, failed to send {}", val)
        }
    }

    match receiver.recv_timeout(Duration::from_millis(1000)) {
        Ok(val) => println!("Received successfully {}", val),
        Err(RecvTimeoutError::Timeout) => println!("Timeout, failed to receive"),
        Err(RecvTimeoutError::Disconnected) => {
            println!("Channel is disconnected, failed to receive")
        }
    }

    match receiver.recv_timeout(Duration::from_millis(1000)) {
        Ok(val) => println!("Received successfully {}", val),
        Err(RecvTimeoutError::Timeout) => println!("Timeout, failed to receive"),
        Err(RecvTimeoutError::Disconnected) => {
            println!("Channel is disconnected, failed to receive")
        }
    }
}

fn select_macro_example() {
    let (s1, r1) = bounded(0);
    let (s2, r2) = bounded(0);
    let (s3, r3) = unbounded();

    let s1_clone = s1.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(500));
        s1_clone.send("Message from s1").unwrap();
    });

    let s2_clone = s2.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        s2_clone.send("Message from s2").unwrap();
    });

    s3.send("Message from s3").unwrap();

    for _ in 0..3 {
        select! {
            recv(r1) -> msg => {
                match msg {
                    Ok(val) => println!("Received from r1: {}", val),
                    Err(_) => println!("r1 is closed"),
                }
            }
            recv(r2) -> msg => {
                match msg {
                    Ok(val) => println!("Received from r2: {}", val),
                    Err(_) => println!("r2 is closed"),
                }
            }
            recv(r3) -> msg => {
                match msg {
                    Ok(val) => println!("Received from r3: {}", val),
                    Err(_) => println!("r3 is closed"),
                }
            }
            default(Duration::from_millis(100)) => {
                println!("No messages received within timeout");
            }
        }
    }
}

fn after_channel_example() {
    println!("start waiting...");

    let timeout = after(Duration::from_secs(2));

    select! {
        recv(timeout) -> _ => {
            println!("2 seconds reached!");
        }
    }

    let timer1 = after(Duration::from_millis(100));
    let timer2 = after(Duration::from_millis(200));
    let timer3 = after(Duration::from_millis(300));

    for _ in 0..3 {
        select! {
            recv(timer1) -> _ => {
                println!("Timer 1 reached!");
            }
            recv(timer2) -> _ => {
                println!("Timer 2 reached!");
            }
            recv(timer3) -> _ => {
                println!("Timer 3 reached!");
            }
        }
    }
}

fn tick_channel_example() {
    let (sender, receiver) = unbounded();

    let ticker = tick(Duration::from_millis(500));

    thread::spawn(move || {
        for i in 0..5 {
            thread::sleep(Duration::from_millis(200));
            sender.send(format!("message {}", i)).unwrap();
        }
    });

    let mut tick_count = 0;
    let mut msg_count = 0;

    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        select! {
            recv(ticker) -> _ => {
                tick_count += 1;
                println!("Tick {}: tick received", tick_count);
            }
            recv(receiver) -> msg => {
                match msg {
                    Ok(val) => {
                        msg_count += 1;
                        println!("Tick {}: received {}", tick_count, val);
                    }
                    Err(_) => {
                        println!("Channel closed");
                        break;
                    }
                }
            }
        }
    }

    println!("Total ticks: {}, Total messages: {}", tick_count, msg_count);
}

fn never_channel_example() {
    let (sender, receiver) = unbounded();
    let never_recv = never::<i32>();

    sender.send(42).unwrap();

    select! {
        recv(receiver) -> msg => {
            match msg {
                Ok(val) => println!("Received from receiver: {}", val),
                Err(_) => println!("Receiver is closed"),
            }
        }
        recv(never_recv) -> msg => {
            println!("This will never happen: {:?}", msg);
        }
        default(Duration::from_millis(100)) => {
            println!("No messages received within timeout");
        }
    }
}

fn main() {
    bounded_channel_example();
    unbounded_channel_example();
    zero_capacity_channel_example();
    non_blocking_operations();
    timeout_operations();
    select_macro_example();
    after_channel_example();
    tick_channel_example();
    never_channel_example();
}
