use ecosystem::hello::{greeter_client::GreeterClient, HelloRequest};
use tonic::Request;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GreeterClient::connect("http://[::1]:50051").await?;

    let req = Request::new(HelloRequest {
        message: "ZhangSan".into(),
    });

    let res = client.say_hello(req).await?;

    println!("{:?}", res);

    Ok(())
}
