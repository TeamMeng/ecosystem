use std::thread;

const X: i32 = 42;

fn main() {
    let x_ref = &X;
    let ref_x_thread = x_ref;
    let ref_x_main = x_ref;

    println!("X ref: {}", x_ref);

    let t1 = thread::spawn(move || {
        println!("In thread: {}", ref_x_thread); // &i32 -> send
    });

    println!("Main thread: {}", ref_x_main);

    t1.join().unwrap();
}
