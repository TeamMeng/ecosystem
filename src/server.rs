use anyhow::Result;
use hello_world::{
    greeter_server::{Greeter, GreeterServer},
    HelloReply, HelloRequest,
};
// use std::time::Duration;
// use tokio::time::sleep;
use tonic::{
    metadata::{Ascii, MetadataValue},
    transport::Server,
    Request, Response, Status,
};

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[derive(Debug, Default)]
pub struct MyGreeter {}

// #[derive(Debug, Clone)]
// struct MyExtension {
//     some_piece_of_data: String,
// }

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        // let extension = request.extensions().get::<MyExtension>().unwrap();

        // println!("extension data = {}", extension.some_piece_of_data);

        let rsp = HelloReply {
            name: format!("Hello {}", request.into_inner().name),
        };

        // let value = request.metadata().get("grpc-timeout").unwrap();
        // println!("{:?}", value);

        // sleep(Duration::from_secs(50)).await;

        // Err(Status::new(Code::InvalidArgument, "参数错误"))
        Ok(Response::new(rsp))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "[::1]:50051".parse().unwrap();
    let greeter = MyGreeter::default();

    // let svc = GreeterServer::with_interceptor(greeter, intercept);

    let greeter = GreeterServer::with_interceptor(greeter, check_auth);

    println!("GreeterServer listening on {}", addr);

    Server::builder()
        .accept_http1(true)
        // .add_service(tonic_web::enable(svc))
        .add_service(greeter)
        .serve(addr)
        .await?;

    Ok(())
}

// fn intercept(mut req: Request<()>) -> Result<Request<()>, Status> {
//     println!("middleware request: {:#?}", req);
//     req.extensions_mut().insert(MyExtension {
//         some_piece_of_data: "foo".to_string(),
//     });
//     Ok(req)
// }

fn check_auth(req: Request<()>) -> Result<Request<()>, Status> {
    let token: MetadataValue<Ascii> = "Bear some-secret-token".parse().unwrap();

    match req.metadata().get("authorization") {
        Some(t) => {
            if token == t {
                println!("{:?}", t);
                println!("auth success");
            }
            Ok(req)
        }
        _ => Err(Status::unauthenticated("No valid auth token")),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn add() {
        assert_eq!(1, 1);
    }
}
