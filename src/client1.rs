// use std::time::Duration;
// use std::time::Duration;
use anyhow::Result;
use hello_world::{greeter_client::GreeterClient, HelloRequest};
use tonic::{metadata::MetadataValue, transport::Channel, Request};
// use tonic::transport::Channel;
// use tower::timeout::Timeout;

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[tokio::main]
async fn main() -> Result<()> {
    // let channel = Channel::from_static("http://[::1]:50051").connect().await?;
    // let timeout_channel = Timeout::new(channel, Duration::from_secs(2));
    // let mut client = GreeterClient::new(timeout_channel);

    // let mut client = GreeterClient::connect("http://[::1]:50051").await?;

    let channel = Channel::from_static("http://[::1]:50051").connect().await?;

    let token: MetadataValue<_> = "Bear some-secret-token".parse()?;

    // let mut client = GreeterClient::with_interceptor(channel, intercept);

    let mut client = GreeterClient::with_interceptor(channel, move |mut req: Request<()>| {
        req.metadata_mut().insert("authorization", token.clone());
        Ok(req)
    });

    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    // request.set_timeout(Duration::from_secs(2));

    let response = client.say_hello(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}

// fn intercept(req: Request<()>) -> Result<Request<()>, Status> {
//     println!("middleware request: {:#?}", req);

//     Ok(req)
// }
