use serde::{Deserialize, Serialize};
use std::{
    env,
    io::{self, Write},
    time::Duration,
};
use tokio::{sync::oneshot, time::interval};
use tokio_stream::StreamExt;

struct Spinner {
    frames: &'static [&'static str],
    frame_idx: usize,
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            frames: &[".", "..", "..."],
            frame_idx: 0,
        }
    }

    fn tick(&mut self) -> io::Result<()> {
        let frame = self.frames[self.frame_idx % self.frames.len()];
        self.frame_idx += 1;
        println!("\r等待模型响应{}", frame);
        io::stdout().flush()
    }

    fn clear_line(&self) -> io::Result<()> {
        println!("\r\x1B[K");
        io::stdout().flush()
    }

    pub async fn run_until(mut self, mut stop_rx: oneshot::Receiver<()>) -> io::Result<()> {
        let mut ticker = interval(tokio::time::Duration::from_millis(120));
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    self.tick()?;
                }
                _ = &mut stop_rx => {
                        self.clear_line()?;
                        break;
                    }
            }
        }
        Ok(())
    }
}

pub fn write_delta(delta: &str) -> io::Result<()> {
    println!("{}", delta);
    io::stdout().flush()
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct RequestBody {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct SseEvent {
    event: String,
    data: String,
    id: Option<String>,
}

#[derive(Default)]
struct SseParser {
    buffer: Vec<u8>,
}

#[derive(Debug, Deserialize)]
struct ChatChunk {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    delta: ChatDelta,
}

#[derive(Debug, Deserialize)]
struct ChatDelta {
    content: Option<String>,
}

impl SseParser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, chunk: &[u8]) -> Vec<SseEvent> {
        self.buffer.extend_from_slice(chunk);
        let mut events = Vec::new();

        while let Some(pos) = find_double_newline(&self.buffer) {
            let raw: Vec<u8> = self.buffer.drain(..pos + 2).collect();
            if let Some(event) = parse_one_event(&raw) {
                events.push(event);
            }
        }

        events
    }
}

fn find_double_newline(buf: &[u8]) -> Option<usize> {
    buf.windows(2).position(|w| w == b"\n\n")
}

fn parse_one_event(raw: &[u8]) -> Option<SseEvent> {
    let text = String::from_utf8_lossy(raw);
    let mut event = SseEvent {
        event: "message".to_string(),
        ..Default::default()
    };
    let mut data_lines: Vec<&str> = Vec::new();
    let mut has_content = false;

    for line in text.lines() {
        if line.starts_with(':') || line.is_empty() {
            continue;
        }
        let (field, value) = match line.split_once(':') {
            Some((f, v)) => (f, v.trim_start()),
            None => (line, ""),
        };
        match field {
            "event" => {
                event.event = value.to_string();
                has_content = true;
            }
            "data" => {
                data_lines.push(value);
                has_content = true;
            }
            "id" => {
                event.id = Some(value.to_string());
                has_content = true;
            }
            _ => {}
        }
    }

    if !has_content {
        return None;
    }

    event.data = data_lines.join("\n");
    Some(event)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let api_key = env::var("APP__API_KEY")?;

    let request_body = RequestBody {
        model: "deepseek-flash".to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: "You are a concise Rust teaching assistant.".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: "What is rust?".to_string(),
            },
        ],
        stream: true,
    };

    let (stop_tx, stop_rx) = oneshot::channel();
    let spinner_handle = tokio::spawn(Spinner::new().run_until(stop_rx));

    let response = reqwest::Client::new()
        .post("https://api.deepseek.com/chat/completions")
        .bearer_auth(api_key)
        .json(&request_body)
        .send()
        .await?;

    let mut stream = response.bytes_stream();
    let mut parser = SseParser::new();
    let mut full_text = String::new();

    while let Some(chunk_result) = stream.next().await {
        let chunk = match chunk_result {
            Ok(chunk) => chunk,
            Err(e) => {
                eprintln!("failed to read chunk: {}", e);
                break;
            }
        };

        for event in parser.push(&chunk) {
            if event.data.trim() == "[DONE]" {
                println!("\n---------- stream finished ----------\n");
                continue;
            }
            match serde_json::from_str::<ChatChunk>(&event.data) {
                Ok(parsed) => {
                    if let Some(delta) =
                        parsed.choices.first().and_then(|c| c.delta.content.clone())
                    {
                        print!("{}", delta);
                        std::io::stdout().flush().unwrap();
                        full_text.push_str(&delta);
                        std::thread::sleep(Duration::from_millis(200));
                    }
                }
                Err(e) => eprintln!("failed to parse chunk: {}", e),
            }
        }
    }

    println!("\nfull assembled text: {}", full_text);

    let _ = stop_tx.send(());
    let _ = spinner_handle.await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_complete_event() {
        let mut parser = SseParser::new();
        let events = parser.push(b"data: hello\n\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "hello");
        assert_eq!(events[0].event, "message");
    }

    #[test]
    fn handles_event_split_across_two_chunks() {
        let mut parser = SseParser::new();
        assert!(parser.push(b"data: hel").is_empty());
        let events = parser.push(b"lo\n\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "hello");
    }

    #[test]
    fn ignores_comment_lines() {
        let mut parser = SseParser::new();
        let events = parser.push(b": keep_alive\n\ndata: real\n\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "real");
    }
}
