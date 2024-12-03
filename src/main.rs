use anyhow::Result;
use axum::http::HeaderMap;
use axum::{routing::get, Router};
use std::env;
use tokio::net::TcpListener;

const ADDR: &str = "0.0.0.0";

async fn index_handler(header_map: HeaderMap) -> String {
    let ret = header_map.get("x-proxy-form");

    if let Some(proxy) = ret {
        match proxy.to_str() {
            Ok(proxy) => {
                let addr = get_addr();
                format!("Up, answered from {} with {}", addr, proxy)
            }
            Err(_) => "to str error".to_string(),
        }
    } else {
        "unknown".to_string()
    }
}

fn get_addr() -> String {
    let port = env::var("PORT").unwrap_or_else(|_| "".to_string());
    format!("{}:{}", ADDR, port)
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = get_addr();

    let listener = TcpListener::bind(addr.clone()).await?;
    println!("Server listening on {}", addr);

    let app = Router::new().route("/health", get(index_handler));

    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn add() {
        assert_eq!(1, 1);
    }
}
