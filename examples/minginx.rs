use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::{
    io::copy,
    net::{TcpListener, TcpStream},
};
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{
    fmt::{format::FmtSpan, Layer},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer as _,
};

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    listen_addr: String,
    upstream_addr: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .pretty()
        .with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let config = resolve_config();

    let config = Arc::new(config);

    info!("Upstream: {}", config.upstream_addr);
    info!("Listen: {}", config.listen_addr);

    let listener = TcpListener::bind(&config.listen_addr).await?;

    loop {
        let (client, raddr) = listener.accept().await?;
        let clone_config = config.clone();
        info!("Accept connection from: {}", raddr);
        tokio::spawn(async move {
            let upstream = TcpStream::connect(&clone_config.upstream_addr).await?;
            proxy(client, upstream).await?;
            Ok::<_, anyhow::Error>(())
        });
    }
}

async fn proxy(mut client: TcpStream, mut upstream: TcpStream) -> Result<()> {
    let (mut client_reader, mut client_writer) = client.split();
    let (mut upstream_reader, mut upstream_writer) = upstream.split();

    let client_to_upstream = copy(&mut client_reader, &mut upstream_writer);
    let upstream_to_client = copy(&mut upstream_reader, &mut client_writer);

    if let Err(e) = tokio::try_join!(client_to_upstream, upstream_to_client) {
        warn!("Error: {}", e);
    }
    Ok(())
}

fn resolve_config() -> Config {
    Config {
        listen_addr: "0.0.0.0:8081".to_string(),
        upstream_addr: "0.0.0.0:8080".to_string(),
    }
}
