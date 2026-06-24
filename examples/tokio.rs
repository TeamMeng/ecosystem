async fn hello() {
    println!("Hello World");
}

async fn run() {
    for i in 0..10 {
        println!("{i}");
    }
}

#[tokio::main]
async fn main() {
    tokio::spawn(run());
    hello().await;
}
