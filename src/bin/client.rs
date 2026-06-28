use std::time::Duration;

use ecosystem::hello::{greeter_client::GreeterClient, HelloRequest};
use tonic::{transport::Channel, Request};
use tower::timeout::Timeout;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let mut client = GreeterClient::connect("http://[::1]:50051").await?;
    let channel = Channel::from_static("http://[::1]:50051").connect().await?;
    let timeout_channel = Timeout::new(channel, Duration::from_secs(2));

    let mut client = GreeterClient::new(timeout_channel);

    let req = Request::new(HelloRequest {
        message: "ZhangSan".into(),
    });

    let res = client.say_hello(req).await?;

    println!("{:?}", res);

    Ok(())
}
