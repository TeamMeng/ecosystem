//! Customer Service Chatbot using Axum + Rig + Ollama
//!
//! This example demonstrates a customer service chatbot for an e-commerce platform
//! ("鸡翅商城") with the following features:
//! - AI-powered responses using Ollama (llama3.2)
//! - Order query tools
//! - Chat memory with session management
//! - Transfer to human agent detection

use anyhow::Result;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use moka::future::Cache;
use rig::{
    client::{CompletionClient, Nothing},
    completion::{Chat, Prompt},
    message::Message,
    tool::Tool,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;
use tracing::{info, instrument, Level};
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;

// ============================================================================
// System Prompt
// ============================================================================

const CUSTOMER_SERVICE_SYSTEM: &str = r#"你是"鸡翅商城"的智能客服助手，代号"鸡翅"。

## 服务范围
- 商品咨询（规格、材质、使用方法）
- 订单查询（状态、物流、预计到达）
- 售后服务（退换货政策、申请流程）
- 账号问题（登录、支付、地址管理）

## 能力边界
- 只回答与鸡翅商城产品和服务相关的问题
- 不提供法律建议、医疗建议或其他专业咨询
- 不评论竞争对手的产品
- 不讨论与购物完全无关的话题（例：帮用户写作业、闲聊）

## 信息准确性原则（重要）
- 退款期限：收货后7天内无理由退换，15天内质量问题退换
- 运费说明：满99元包邮，不足99元收12元运费
- 如果用户询问具体订单信息，告知需要查询系统，请提供订单号
- 对于不确定的信息，说"我帮您查一下"或"建议联系人工客服确认"，不要猜测

## 转人工条件（遇到以下情况必须主动提出转人工）
- 用户明确表示不满意或投诉
- 涉及金额较大的纠纷（超过500元）
- 连续2轮没有解决用户问题
- 用户情绪激动（多次使用感叹号、出现"投诉"、"曝光"、"消费者协会"等词）
- 需要查询实时订单/物流状态（系统没有集成实时查询时）

## 回复规范
- 称呼用户为"您"
- 回复控制在150字以内，简洁直接
- 语气：专业、友好、有温度，不要过于机械
- 不要在每条回复末尾都加"还有其他问题吗"（太机械）

## 禁止行为
- 不得承诺系统权限以外的优惠或补偿
- 不得透露其他用户的订单信息
- 不得用"我不知道"结束对话，应给出下一步引导"#;

// ============================================================================
// Database Models
// ============================================================================

#[derive(Debug, Clone, sqlx::FromRow)]
struct Order {
    id: String,
    status: String,
    logistics_info: Option<String>,
    expected_delivery: Option<String>,
    created_at: String,
}

#[derive(Debug, Clone)]
struct ChatSession {
    #[allow(dead_code)]
    id: String,
    history: Vec<Message>,
    #[allow(dead_code)]
    created_at: DateTime<Utc>,
}

impl ChatSession {
    fn new(id: String) -> Self {
        Self {
            id,
            history: Vec::new(),
            created_at: Utc::now(),
        }
    }
}

// ============================================================================
// Order Query Tools
// ============================================================================

#[derive(Debug, Clone)]
struct OrderQueryTools {
    pool: PgPool,
}

impl OrderQueryTools {
    fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
enum ToolError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Order not found")]
    NotFound,
}

impl From<ToolError> for String {
    fn from(err: ToolError) -> Self {
        err.to_string()
    }
}

impl Tool for OrderQueryTools {
    const NAME: &'static str = "order_query";

    type Error = ToolError;
    type Args = serde_json::Value;
    type Output = String;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: "order_query".to_string(),
            description: "综合订单查询工具，可根据参数类型自动识别操作：查询订单状态、查询商品信息、申请售后退换货".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "description": "操作类型：query_status（查询订单）/ query_product（查询商品）/ create_aftersale（申请售后）"
                    },
                    "order_id": {
                        "type": "string",
                        "description": "订单号，格式如 ORD20240101001"
                    },
                    "product_query": {
                        "type": "string",
                        "description": "商品ID或商品名称"
                    },
                    "aftersale_type": {
                        "type": "string",
                        "description": "售后类型：REFUND（退款）/ EXCHANGE（换货）/ REPAIR（维修）"
                    },
                    "description": {
                        "type": "string",
                        "description": "问题描述"
                    }
                },
                "required": ["action"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let action = args["action"].as_str().unwrap_or("");

        match action {
            "query_status" => {
                let order_id = args["order_id"].as_str().unwrap_or("");
                self.query_order_status(order_id).await
            }
            "query_product" => {
                let product_query = args["product_query"].as_str().unwrap_or("");
                self.query_product(product_query).await
            }
            "create_aftersale" => {
                let order_id = args["order_id"].as_str().unwrap_or("");
                let aftersale_type = args["aftersale_type"].as_str().unwrap_or("REFUND");
                let description = args["description"].as_str().unwrap_or("");
                self.create_aftersale(order_id, aftersale_type, description)
                    .await
            }
            _ => Ok(
                "未知操作类型，请指定 action：query_status、query_product 或 create_aftersale"
                    .to_string(),
            ),
        }
    }
}

impl OrderQueryTools {
    async fn query_order_status(&self, order_id: &str) -> Result<String, ToolError> {
        let order = sqlx::query_as::<_, Order>(
            "SELECT id, status, logistics_info, expected_delivery, created_at FROM orders WHERE id = $1",
        )
        .bind(order_id)
        .fetch_optional(&self.pool)
        .await?;

        match order {
            Some(o) => Ok(format!(
                "订单号：{}\n状态：{}\n下单时间：{}\n预计到达：{}\n物流信息：{}",
                o.id,
                o.status,
                o.created_at,
                o.expected_delivery.unwrap_or_else(|| "未设置".to_string()),
                o.logistics_info
                    .unwrap_or_else(|| "暂无物流信息".to_string())
            )),
            None => Ok("未找到订单，请确认订单号是否正确".to_string()),
        }
    }

    async fn query_product(&self, _product_query: &str) -> Result<String, ToolError> {
        // Mock product data - in production, query product database
        Ok("商品：XX耳机，当前价格：299元，库存：有货，颜色：黑/白/红".to_string())
    }

    async fn create_aftersale(
        &self,
        order_id: &str,
        aftersale_type: &str,
        description: &str,
    ) -> Result<String, ToolError> {
        let ticket_id = format!("AS{}", Utc::now().timestamp_millis());

        // In production, insert aftersale record to database
        info!(
            "Created aftersale ticket: {} for order: {}, type: {}, description: {}",
            ticket_id, order_id, aftersale_type, description
        );

        Ok(format!(
            "售后申请已提交，工单号：{}\n预计24小时内处理，处理结果会通过短信通知您",
            ticket_id
        ))
    }
}

// ============================================================================
// Chat Service
// ============================================================================

struct ChatService {
    session_cache: Cache<String, Arc<RwLock<ChatSession>>>,
    tools: OrderQueryTools,
}

impl ChatService {
    fn new(pool: PgPool) -> Self {
        let tools = OrderQueryTools::new(pool);

        Self {
            session_cache: Cache::builder().max_capacity(10000).build(),
            tools,
        }
    }

    async fn get_or_create_session(&self, session_id: &str) -> Arc<RwLock<ChatSession>> {
        if let Some(session) = self.session_cache.get(session_id).await {
            return session;
        }

        let session = Arc::new(RwLock::new(ChatSession::new(session_id.to_string())));
        self.session_cache
            .insert(session_id.to_string(), session.clone())
            .await;
        session
    }

    #[instrument(skip(self))]
    async fn chat(&self, session_id: &str, user_message: &str) -> Result<ChatResponse> {
        let session = self.get_or_create_session(session_id).await;
        let mut session_guard = session.write().await;

        // Add user message to history
        session_guard.history.push(Message::user(user_message));

        // Build agent using Ollama
        let client = rig::providers::ollama::Client::new(Nothing).unwrap();
        let tools = self.tools.clone();
        let agent = client
            .agent("llama3.2")
            .preamble(CUSTOMER_SERVICE_SYSTEM)
            .tool(tools)
            .build();

        // Get chat history for context (last 20 messages)
        let history: Vec<Message> = session_guard
            .history
            .iter()
            .rev()
            .take(20)
            .cloned()
            .collect();
        let history: Vec<_> = history.into_iter().rev().collect();

        // Call the agent - use chat for conversation with history
        let response: String = if history.is_empty() {
            agent.prompt(user_message).await?.to_string()
        } else {
            agent.chat(user_message, history).await?.to_string()
        };

        // Add assistant response to history
        session_guard.history.push(Message::assistant(&response));

        // Check if transfer to human is needed
        let needs_human = should_transfer_to_human(user_message, &response);

        Ok(ChatResponse {
            reply: response,
            needs_human_transfer: needs_human,
            session_id: session_id.to_string(),
        })
    }
}

fn should_transfer_to_human(user_message: &str, bot_response: &str) -> bool {
    let escalation_keywords = ["投诉", "曝光", "消费者协会", "315", "退款不处理"];

    let has_escalation = escalation_keywords
        .iter()
        .any(|kw| user_message.contains(kw));

    let bot_suggests_transfer =
        bot_response.contains("人工客服") || bot_response.contains("人工为您");

    has_escalation || bot_suggests_transfer
}

#[derive(Debug, Serialize)]
struct ChatResponse {
    reply: String,
    #[serde(rename = "needsHumanTransfer")]
    needs_human_transfer: bool,
    #[serde(rename = "sessionId")]
    session_id: String,
}

// ============================================================================
// API Types
// ============================================================================

#[derive(Debug, Deserialize)]
struct ChatRequest {
    message: String,
    #[serde(rename = "sessionId", default)]
    session_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct ChatApiResponse {
    reply: String,
    #[serde(rename = "transferToHuman")]
    transfer_to_human: bool,
    #[serde(rename = "sessionId")]
    session_id: String,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

// ============================================================================
// API Handlers
// ============================================================================

async fn chat_handler(
    State(service): State<Arc<ChatService>>,
    Json(request): Json<ChatRequest>,
) -> axum::response::Response {
    let session_id = request
        .session_id
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    match service.chat(&session_id, &request.message).await {
        Ok(response) => {
            let api_response = ChatApiResponse {
                reply: response.reply,
                transfer_to_human: response.needs_human_transfer,
                session_id: response.session_id,
            };
            (StatusCode::OK, Json(api_response)).into_response()
        }
        Err(e) => {
            tracing::error!("Chat error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "处理消息失败，请稍后重试".to_string(),
                }),
            )
                .into_response()
        }
    }
}

async fn end_session_handler(
    State(service): State<Arc<ChatService>>,
    Query(session_id): Query<String>,
) -> impl IntoResponse {
    // Remove session from cache
    service.session_cache.invalidate(&session_id).await;
    (StatusCode::OK, Json(json!({"message": "会话已结束"})))
}

async fn health_handler() -> impl IntoResponse {
    Json(json!({"status": "healthy"}))
}

// ============================================================================
// App State
// ============================================================================

fn create_app(state: Arc<ChatService>) -> Router {
    Router::new()
        .route("/api/customer-service/chat", post(chat_handler))
        .route(
            "/api/customer-service/end-session",
            post(end_session_handler),
        )
        .route("/health", get(health_handler))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(true)
        .with_thread_ids(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting Customer Service Chat Server...");

    // Database connection pool
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/postgres".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    info!("Connected to database");

    // Verify Ollama is running
    info!("Checking Ollama connection...");
    let _client = rig::providers::ollama::Client::new(Nothing).unwrap();
    info!("Ollama client initialized (using llama3.2 model)");

    // Create chat service
    let service = Arc::new(ChatService::new(pool));

    // Create app
    let app = create_app(service);

    // Start server
    let addr = "0.0.0.0:8080";
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_transfer_to_human_escalation() {
        let user_msg = "我要投诉，你们产品太差了";
        let bot_response = "我理解您的不满...";
        assert!(should_transfer_to_human(user_msg, bot_response));
    }

    #[test]
    fn test_should_transfer_to_human_bot_suggests() {
        let user_msg = "什么时候发货";
        let bot_response = "建议您联系人工客服查询";
        assert!(should_transfer_to_human(user_msg, bot_response));
    }

    #[test]
    fn test_should_not_transfer() {
        let user_msg = "我想查一下订单状态";
        let bot_response = "好的，请提供订单号";
        assert!(!should_transfer_to_human(user_msg, bot_response));
    }

    #[test]
    fn test_chat_session_new() {
        let session = ChatSession::new("test-123".to_string());
        assert_eq!(session.id, "test-123");
        assert!(session.history.is_empty());
    }
}
