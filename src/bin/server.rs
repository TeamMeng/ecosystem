use ecosystem::hello::{
    greeter_server::{Greeter, GreeterServer},
    HelloReply, HelloRequest,
};
use tonic::{transport::Server, Code, Request, Response, Status};

#[derive(Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        _request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Hello World");

        let _rsp = HelloReply {
            message: "Hello".to_string(),
        };

        Err(Status::new(Code::InvalidArgument, "参数错误"))

        // Ok(Response::new(rsp))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let g = MyGreeter::default();

    Server::builder()
        .add_service(GreeterServer::new(g))
        .serve(addr)
        .await?;

    Ok(())
}
