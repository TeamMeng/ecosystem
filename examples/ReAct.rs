use anyhow::Result;
use axum::{routing::post, Json, Router};
use chrono::Local;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, net::SocketAddr};

type ToolFn = fn(String) -> String;

#[derive(Clone)]
struct SimpleReActAgent {
    tools: HashMap<String, ToolFn>,
    http_client: Client,
}

#[derive(Debug, Deserialize)]
struct TaskRequest {
    task: String,
}

#[derive(Debug, Serialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

// curl -X POST http://localhost:8080/api/agent/react \
//                                      -H "Content-Type: application/json" \
//                                      -d '{"task":"上海今天的天气怎么样？"}'
// 上海今天的天气是晴朗的，气温在8到15摄氏度之间，并且会有轻微的北风。⏎

#[tokio::main]
async fn main() -> Result<()> {
    let agent = SimpleReActAgent::new();

    let app = Router::new()
        .route("/api/agent/react", post(run_agent))
        .with_state(agent);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    println!("Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;
    Ok(())
}

async fn run_agent(
    axum::extract::State(agent): axum::extract::State<SimpleReActAgent>,
    Json(request): Json<TaskRequest>,
) -> String {
    agent.run(request.task, 10).await
}

impl SimpleReActAgent {
    fn new() -> Self {
        let mut tools: HashMap<String, ToolFn> = HashMap::new();

        tools.insert("getWeather".to_string(), get_weather);
        tools.insert("getDate".to_string(), get_date);

        Self {
            tools,
            http_client: Client::new(),
        }
    }

    async fn run(&self, user_task: String, max_iterations: usize) -> String {
        let mut messages = vec![ChatMessage {
            role: "user".to_string(),
            content: user_task,
        }];

        let system_prompt = self.build_system_prompt();

        for i in 0..max_iterations {
            let model_output = match self.call_ollama(&system_prompt, &messages).await {
                Ok(output) => output,
                Err(err) => return format!("调用 Ollama 失败: {}", err),
            };

            messages.push(ChatMessage {
                role: "assistant".to_string(),
                content: model_output.clone(),
            });

            println!("=== 第 {} 轮 ===", i + 1);
            println!("{}", model_output);

            if model_output.contains("Final Answer:") {
                return extract_final_answer(&model_output);
            }

            let tool_name = match extract_action(&model_output) {
                Some(name) => name,
                None => return "模型输出格式异常，无法继续执行".to_string(),
            };

            let tool_input = extract_action_input(&model_output);

            let observation = match self.tools.get(&tool_name) {
                Some(tool) => tool(tool_input),
                None => format!("工具 {} 不存在，请换一个", tool_name),
            };

            println!("Observation: {}", observation);

            messages.push(ChatMessage {
                role: "user".to_string(),
                content: format!("Observation: {}", observation),
            });
        }

        format!("超过最大迭代次数（{}），任务未完成", max_iterations)
    }

    async fn call_ollama(
        &self,
        system_prompt: &str,
        messages: &[ChatMessage],
    ) -> Result<String, reqwest::Error> {
        let mut ollama_messages = vec![ChatMessage {
            role: "system".to_string(),
            content: system_prompt.to_string(),
        }];

        ollama_messages.extend_from_slice(messages);

        let body = json!({
            "model": "qwen2.5:7b",
            "stream": false,
            "messages": ollama_messages,
            "options": {
                "temperature": 0.2
            }
        });

        let resp: serde_json::Value = self
            .http_client
            .post("http://localhost:11434/api/chat")
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        let content = resp["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(content)
    }

    fn build_system_prompt(&self) -> String {
        r#"
你是一个智能助手，按照以下格式严格输出，每次只做一个动作。

可用工具：
- getWeather(input: JSON {"city": "城市名", "date": "today/tomorrow"})：查询天气
- getDate(input: 无)：获取今天的日期

输出格式必须严格如下：

Thought: [你的分析和下一步计划]
Action: [工具名]
Action Input: [工具参数，JSON 格式]

收到 Observation 后继续思考，直到可以回答为止：

Thought: [分析观察结果]
Final Answer: [给用户的最终回答]

注意：
- 每次只输出一个 Action 或 Final Answer
- 不要输出 Markdown
- 不要输出多余解释
- 工具名必须和工具列表完全一致
"#
        .to_string()
    }
}

fn get_weather(_input: String) -> String {
    r#"{"city": "上海", "weather": "晴", "temp": "8~15°C", "wind": "北风3级"}"#.to_string()
}

fn get_date(_input: String) -> String {
    Local::now().date_naive().to_string()
}

fn extract_final_answer(output: &str) -> String {
    output
        .split_once("Final Answer:")
        .map(|(_, answer)| answer.trim().to_string())
        .unwrap_or_else(|| output.to_string())
}

fn extract_action(output: &str) -> Option<String> {
    output
        .lines()
        .find(|line| line.starts_with("Action:"))
        .map(|line| line["Action:".len()..].trim().to_string())
}

fn extract_action_input(output: &str) -> String {
    output
        .lines()
        .find(|line| line.starts_with("Action Input:"))
        .map(|line| line["Action Input:".len()..].trim().to_string())
        .unwrap_or_default()
}
