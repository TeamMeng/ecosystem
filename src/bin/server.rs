use ecosystem::hello::{
    greeter_server::{Greeter, GreeterServer},
    HelloReply, HelloRequest,
};
use tonic::{transport::Server, Request, Response, Status};

// use std::{thread::sleep, time::Duration};

#[derive(Default)]
pub struct MyGreeter {}

#[derive(Clone)]
pub struct MyExtension {
    some_piece_of_data: String,
}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        let extension = request.extensions().get::<MyExtension>().unwrap();
        println!("extension data = {}", extension.some_piece_of_data);

        // println!("Hello World");

        let rsp = HelloReply {
            message: "Hello".to_string(),
        };

        println!("{:#?}", request);
        let value = request.metadata().get("grpc-timeout").unwrap();
        println!("{:?}", value);

        // Err(Status::new(Code::InvalidArgument, "参数错误"))

        Ok(Response::new(rsp))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let g = MyGreeter::default();

    Server::builder()
        .add_service(GreeterServer::with_interceptor(g, intercept))
        .serve(addr)
        .await?;

    Ok(())
}

#[allow(clippy::result_large_err)]
fn intercept(mut req: Request<()>) -> Result<Request<()>, Status> {
    println!("middleware request: {:#?}", req);

    req.extensions_mut().insert(MyExtension {
        some_piece_of_data: "foo".to_string(),
    });

    Ok(req)
}
