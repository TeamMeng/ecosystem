use anyhow::Context;
use ecosystem::MyError;
use std::fs;

fn main() -> Result<(), anyhow::Error> {
    println!("size of MyError is {}", size_of::<MyError>());
    println!("size of anyhow::Error is {}", size_of::<anyhow::Error>());
    println!("size of std::io::Error is {}", size_of::<std::io::Error>());

    println!(
        "size of std::num::ParseIntError is {}",
        size_of::<std::num::ParseIntError>()
    );

    println!(
        "size of serde_json::Error is {}",
        size_of::<serde_json::Error>()
    );

    println!("size of string is {}", size_of::<String>());

    let filename = "non-existent-file.txt";
    let _ = fs::File::open("").with_context(|| format!("Can not find {}", filename))?;

    fail_with_error()?;
    Ok(())
}

fn fail_with_error() -> Result<(), MyError> {
    Err(MyError::Custom("This is a custom error".to_string()))
}
