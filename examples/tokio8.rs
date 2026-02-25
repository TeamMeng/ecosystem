use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);

    stdout.write_all("=== simple op ===\n".as_bytes()).await?;
    stdout
        .write_all("please input op, as 2 + 3\n".as_bytes())
        .await?;
    stdout.write_all("input quit out\n".as_bytes()).await?;

    loop {
        let mut input = String::new();
        reader.read_line(&mut input).await?;
        let input = input.trim();

        if input == "quit" {
            stdout.write_all("bye\n".as_bytes()).await?;
            break;
        }

        match evaluate_expression(input) {
            Ok(result) => {
                stdout
                    .write_all(format!("ret: {}\n", result).as_bytes())
                    .await?
            }
            Err(e) => {
                stdout
                    .write_all(format!("error: {}\n", e).as_bytes())
                    .await?
            }
        }
    }
    Ok(())
}

fn evaluate_expression(expr: &str) -> Result<f64, String> {
    let parts: Vec<&str> = expr.split_whitespace().collect();

    if parts.len() != 3 {
        return Err("格式错误，请使用：数字 运算符 数字".to_string());
    }

    let a: f64 = parts[0].parse().map_err(|_| "第一个数字无效")?;
    let op = parts[1];
    let b: f64 = parts[2].parse().map_err(|_| "第一个数字无效")?;

    match op {
        "+" => Ok(a + b),
        "-" => Ok(a - b),
        "*" => Ok(a * b),
        "/" => {
            if b == 0.0 {
                Err("除零错误".to_string())
            } else {
                Ok(a / b)
            }
        }
        _ => Err(format!("不支持的运算符: {}", op)),
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use std::time::Instant;
    use tokio::{
        fs::{self, File},
        io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    };
    use tokio_stream::StreamExt;
    use tokio_util::io::ReaderStream;

    #[tokio::test]
    async fn file_open_should_work() -> Result<()> {
        let mut f = File::open("./Cargo.toml").await?;
        let mut buffer = [0; 1024];

        let n = f.read(&mut buffer).await?;
        println!("read bytes: {:?}", &buffer[..n]);

        Ok(())
    }

    #[tokio::test]
    async fn file_create_should_work() -> Result<()> {
        let mut f = File::create("output.txt").await?;
        f.write_all(b"Hello, world!").await?;
        f.flush().await?;

        fs::remove_file("output.txt").await?;
        Ok(())
    }

    #[tokio::test]
    async fn file_copy_should_work() -> Result<()> {
        async fn copy_file(src: &str, dst: &str) -> Result<u64> {
            let mut src_file = File::open(src).await?;
            let mut dst_file = File::create(dst).await?;

            let mut buffer = [0; 8192];
            let mut total_bytes = 0;

            loop {
                let bytes_read = src_file.read(&mut buffer).await?;
                if bytes_read == 0 {
                    break;
                }

                dst_file.write_all(&buffer[..bytes_read]).await?;
                total_bytes += bytes_read;
            }
            dst_file.flush().await?;
            Ok(total_bytes.try_into()?)
        }

        match copy_file("Cargo.toml", "Cargo-copy.toml").await {
            Ok(bytes) => println!("copy success: {} bytes", bytes),
            Err(e) => println!("copy failed: {}", e),
        }

        fs::remove_file("Cargo-copy.toml").await?;
        Ok(())
    }

    #[tokio::test]
    async fn bufreader_should_work() -> Result<()> {
        let f = File::open("Cargo.toml").await?;
        let mut reader = BufReader::new(f);
        let mut buffer = String::new();

        while reader.read_line(&mut buffer).await? > 0 {
            print!("{}", buffer);
            buffer.clear();
        }

        let f = File::open("Cargo.toml").await?;
        let reader = BufReader::new(f);
        let mut lines = reader.lines();

        let mut line_count = 0;
        while let Some(line) = lines.next_line().await? {
            line_count += 1;
            println!("第 {} 行: {}", line_count, line);
        }

        println!("total read {}", line_count);
        Ok(())
    }

    #[tokio::test]
    async fn bufwriter_should_work() -> Result<()> {
        let f = File::create("output.txt").await?;
        {
            let mut writer = BufWriter::new(f);

            writer.write(&[42u8]).await?;
            writer.write_all(b"Hello, world!").await?;

            writer.flush().await?;
        }

        fs::remove_file("output.txt").await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_for_unbuffered_buffered() -> Result<()> {
        let start = Instant::now();
        {
            let mut file = File::create("unbuffered.txt").await?;
            for i in 0..10000 {
                file.write_all(format!("Line {}\n", i).as_bytes()).await?;
            }
            file.flush().await?;
            fs::remove_file("unbuffered.txt").await?;
        }
        let unbuffered_time = start.elapsed();

        let start = Instant::now();
        {
            let file = File::create("buffered.txt").await?;
            let mut writer = BufWriter::new(file);
            for i in 0..10000 {
                writer.write_all(format!("Line {}\n", i).as_bytes()).await?;
            }
            writer.flush().await?;
            fs::remove_file("buffered.txt").await?;
        }
        let buffered_time = start.elapsed();

        println!("Unbuffered time: {:?}", unbuffered_time);
        println!("Buffered time: {:?}", buffered_time);
        println!(
            "rise up: {}",
            unbuffered_time.as_secs_f64() / buffered_time.as_secs_f64()
        );

        Ok(())
    }

    #[tokio::test]
    async fn async_read_and_write_to_stream_and_sink() -> Result<()> {
        let file = File::open("Cargo.toml").await?;
        let mut stream = ReaderStream::new(file);

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(bytes) => println!("read {:?} bytes", bytes),
                Err(e) => println!("read err: {}", e),
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn io_should_work() -> Result<()> {
        let mut stdout = io::stdout();
        stdout.write_all(b"name: ").await?;
        stdout.flush().await?;

        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut input = String::new();

        reader.read_line(&mut input).await?;

        let name = input.trim();

        stdout
            .write_all(format!("xingxing, {}\n", name).as_bytes())
            .await?;
        stdout.flush().await?;

        Ok(())
    }
}
