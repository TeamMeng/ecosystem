use anyhow::Result;
use dashmap::DashMap;
use futures::{stream::SplitStream, SinkExt, StreamExt};
use std::{fmt::Display, net::SocketAddr, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{
    fmt::{format::FmtSpan, Layer},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer as _,
};

const MAX_MESSAGES: usize = 128;

#[derive(Debug, Default)]
struct State {
    peers: DashMap<SocketAddr, mpsc::Sender<Arc<Message>>>,
}

#[derive(Debug)]
struct Peer {
    username: String,
    stream: SplitStream<Framed<TcpStream, LinesCodec>>,
}

#[derive(Debug)]
enum Message {
    UserJoined(String),
    UserLeft(String),
    Chat { sender: String, content: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .pretty()
        .with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    console_subscriber::init();

    let addr = "0.0.0.0:8080";
    info!("Server listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;

    let state = Arc::new(State::default());

    loop {
        let (stream, raddr) = listener.accept().await?;
        let state_clone = state.clone();
        info!("Accepted connection from: {}", raddr);
        tokio::spawn(async move {
            if let Err(e) = handle_client(state_clone, raddr, stream).await {
                warn!("Failed to handle client {}: {}", raddr, e);
            }
        });
    }
}

async fn handle_client(state: Arc<State>, raddr: SocketAddr, stream: TcpStream) -> Result<()> {
    let mut stream = Framed::new(stream, LinesCodec::new());
    stream.send("Enter your username:").await?;

    let username = match stream.next().await {
        Some(Ok(username)) => username,
        Some(Err(e)) => return Err(e.into()),
        None => return Ok(()),
    };

    let mut peer = state.add(raddr, username, stream).await;

    // notify others that a new user has joined
    let message = Arc::new(Message::user_joined(&peer.username));

    info!("{}", message);

    state.broadcast(raddr, message).await;

    while let Some(line) = peer.stream.next().await {
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                warn!("Failed to read line from {}: {}", raddr, e);
                break;
            }
        };

        let message = Arc::new(Message::chat(&peer.username, line));

        state.broadcast(raddr, message).await;
    }
    // remove peer from state
    state.peers.remove(&raddr);

    // when while loop exit, peer has left the chat or line reading failed
    let message = Arc::new(Message::user_left(&peer.username));
    info!("{}", message);

    state.broadcast(raddr, message).await;

    Ok(())
}

impl State {
    async fn broadcast(&self, addr: SocketAddr, message: Arc<Message>) {
        for peer in self.peers.iter() {
            if peer.key() != &addr {
                if let Err(e) = peer.value().send(message.clone()).await {
                    warn!("Failed to send message to {}: {}", peer.key(), e);
                    // if send failed, peer might be gone, remove peer from state
                    self.peers.remove(peer.key());
                }
            }
        }
    }

    async fn add(
        &self,
        addr: SocketAddr,
        username: String,
        stream: Framed<TcpStream, LinesCodec>,
    ) -> Peer {
        let (tx, mut rx) = mpsc::channel(MAX_MESSAGES);
        self.peers.insert(addr, tx);

        let (mut stream_sender, stream_receiver) = stream.split();

        // receive messages from others, and send them to the client
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = stream_sender.send(message.to_string()).await {
                    warn!("Failed to send message to {}: {}", addr, e);
                    break;
                }
            }
        });
        // return peer
        Peer {
            username,
            stream: stream_receiver,
        }
    }
}

impl Message {
    fn user_joined(username: &str) -> Self {
        let content = format!("{} has joined the chat", username);
        Self::UserJoined(content)
    }

    fn user_left(username: &str) -> Self {
        let content = format!("{} has left the chat", username);
        Self::UserLeft(content)
    }

    fn chat(sender: impl Into<String>, content: impl Into<String>) -> Self {
        Self::Chat {
            sender: sender.into(),
            content: content.into(),
        }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::UserJoined(content) => writeln!(f, "[ {} ]", content),
            Message::UserLeft(content) => writeln!(f, "[{} :(]", content),
            Message::Chat { sender, content } => writeln!(f, "{}, {}", sender, content),
        }
    }
}
