//! RabbitMQ 生产者示例
//!
//! 该示例演示了如何连接 RabbitMQ、声明交换机、构建消息并发布到交换机。
//! 运行前需要确保 RabbitMQ 服务已启动。

use anyhow::Result;
use lapin::{
    options::{BasicPublishOptions, ExchangeDeclareOptions},
    types::{FieldTable, ShortString},
    BasicProperties, Connection, ConnectionProperties, ExchangeKind,
};
use serde::{Deserialize, Serialize};

/// 聊天事件结构体，用于序列化消息内容
#[derive(Debug, Serialize, Deserialize)]
struct ChatEvent {
    /// 事件类型
    event_type: String,
    /// 用户 ID
    user_id: i64,
    /// 消息 ID
    message: i64,
    /// 消息内容
    content: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 从环境变量获取 RabbitMQ 连接地址，默认连接到本地
    let addr =
        std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".to_string());

    // 创建与 RabbitMQ 的连接
    let conn = Connection::connect(&addr, ConnectionProperties::default()).await?;
    // 在连接上创建一个通道，通道是执行操作的主要单位
    let channel = conn.create_channel().await?;

    // 声明一个名为 "chat_events" 的 Topic 类型交换机
    // durable: true 表示交换机在服务器重启后仍然存在
    channel
        .exchange_declare(
            ShortString::from("chat_events"),
            ExchangeKind::Topic,
            ExchangeDeclareOptions {
                durable: true,
                auto_delete: false, // auto_delete: false 表示交换机不会在没有绑定队列时自动删除
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;

    // 构造一个待发送的聊天事件
    let event = ChatEvent {
        event_type: "chat.message.created".to_string(),
        user_id: 1001,
        message: 421,
        content: "Hello, world!".to_string(),
    };

    // 将事件序列化为 JSON 字节数组
    let payload = serde_json::to_vec(&event)?;

    // 发布消息到 "chat_events" 交换机，使用路由键 "chat.message.created"
    // mandatory: true 表示消息必须被正确路由到队列，否则返回给生产者
    // immediate: false 表示不要求立即投递，不会在消费者不可用时缓存消息
    channel
        .basic_publish(
            ShortString::from("chat_events"),
            ShortString::from("chat.message.created"),
            BasicPublishOptions {
                mandatory: true,
                immediate: false,
            },
            &payload,
            // 设置消息内容类型为 JSON
            BasicProperties::default().with_content_type("application/json".into()),
        )
        .await?
        // 等待消息确认，确保消息已成功发布
        .await?;

    println!("Published message: {:?}", event);
    Ok(())
}
