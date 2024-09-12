use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, PartialEq, Eq, Clone, Deserialize)]
struct User {
    name: String,
    age: u32,
    dob: DateTime<Utc>,
    skills: Vec<String>,
}

fn main() -> Result<()> {
    let user = User {
        name: "Alice".to_string(),
        age: 30,
        dob: Utc::now(),
        skills: vec!["Rust".to_string(), "Python".to_string()],
    };

    let json = serde_json::to_string(&user)?;
    println!("{}", json);

    let ret: User = serde_json::from_str(&json)?;
    println!("{:?}", ret);
    assert_eq!(ret, user);

    Ok(())
}
