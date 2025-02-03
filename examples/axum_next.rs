use anyhow::Result;
use axum::{
    extract::Request,
    middleware::{from_fn, Next},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use http::{Method, StatusCode};
use tokio::{net::TcpListener, time::Instant};

#[tokio::main]
async fn main() -> Result<()> {
    let app = Router::new()
        .route("/", get(index_handler))
        .layer(from_fn(auth));

    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    axum::serve(listener, app).await?;

    Ok(())
}

async fn auth(request: Request, next: Next) -> Response {
    if request.method() == Method::GET {
        let start = Instant::now();
        let mut res = next.run(request).await;

        let elapsed = format!("{}us", start.elapsed().as_micros());

        res.headers_mut()
            .insert("server-time", elapsed.parse().unwrap());

        res
    } else {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

async fn index_handler() -> &'static str {
    "Hello"
}
