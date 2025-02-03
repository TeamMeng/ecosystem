use anyhow::Result;
use axum::{routing::post, Json, Router};
use serde::Deserialize;
use tokio::net::TcpListener;
use validator::{Validate, ValidationError};

#[allow(dead_code)]
#[derive(Debug, Deserialize, Validate)]
struct User {
    #[validate(length(min = 2), custom(function = "validate_name"))]
    name: String,

    #[validate(range(min = 18, max = 20, message = "请输入正确范围"))]
    age: u8,

    #[validate(email)]
    email: String,

    #[serde(rename = "my_address")]
    address: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = Router::new().route("/hello_user", post(hello_user_handler));

    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    axum::serve(listener, app).await?;

    Ok(())
}

async fn hello_user_handler(Json(data): Json<User>) -> String {
    if let Err(e) = data.validate() {
        return format!("msg {}", e);
    }
    format!("Hello {:?}", data)
}

fn validate_name(name: &str) -> Result<(), ValidationError> {
    if name == "admin" {
        return Err(ValidationError::new("参数错误"));
    }
    Ok(())
}
