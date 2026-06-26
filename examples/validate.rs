use anyhow::Result;
use axum::{routing::post, Json, Router};
use serde::Deserialize;
use tokio::net::TcpListener;
use validator::ValidateEmail;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct User {
    name: String,

    age: u8,

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
    if let Err(e) = validate_user(&data) {
        return format!("msg {}", e);
    }
    format!("Hello {:?}", data)
}

fn validate_user(user: &User) -> Result<(), &'static str> {
    if user.name.chars().count() < 2 {
        return Err("name length must be at least 2");
    }
    if user.name == "admin" {
        return Err("参数错误");
    }
    if !(18..=20).contains(&user.age) {
        return Err("请输入正确范围");
    }
    if !user.email.validate_email() {
        return Err("email is invalid");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_user() {
        let user = User {
            name: "a".to_string(),
            age: 21,
            email: "bad".to_string(),
            address: "x".to_string(),
        };

        assert!(validate_user(&user).is_err());
    }
}
