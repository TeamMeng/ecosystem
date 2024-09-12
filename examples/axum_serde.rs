use anyhow::Result;
use axum::{
    extract::State,
    routing::{get, patch},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tracing::{info, instrument, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::{format::FmtSpan, Layer},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer as _,
};

#[derive(Debug, Serialize, PartialEq, Eq, Clone)]
struct User {
    name: String,
    age: u32,
    dob: DateTime<Utc>,
    skills: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct UserUpdate {
    age: Option<u32>,
    skills: Option<Vec<String>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .pretty()
        .with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await?;
    info!("Server listening on {}", addr);

    let user = User {
        name: "Alice".to_string(),
        age: 30,
        dob: Utc::now(),
        skills: vec!["Rust".to_string(), "Python".to_string()],
    };
    let user = Arc::new(Mutex::new(user));

    let app = Router::new()
        .route("/", get(user_handler))
        .route("/", patch(update_handler))
        .with_state(user.clone());

    axum::serve(listener, app).await?;

    Ok(())
}

#[instrument]
async fn user_handler(State(user): State<Arc<Mutex<User>>>) -> Json<User> {
    let user = user.lock().unwrap().clone();
    user.into()
}

#[instrument]
async fn update_handler(
    State(state): State<Arc<Mutex<User>>>,
    Json(user_update): Json<UserUpdate>,
) -> Json<User> {
    let mut user = state.lock().unwrap();

    if let Some(age) = user_update.age {
        user.age = age;
    }

    if let Some(skills) = user_update.skills {
        user.skills = skills;
    }
    (*user).clone().into()
}
