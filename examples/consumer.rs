//! RabbitMQ 消费者示例
//!
//! 该示例演示了如何连接 RabbitMQ、声明交换机和队列、绑定队列到交换机，
//! 并消费消息。运行前需要确保 RabbitMQ 服务已启动。

use anyhow::Result;
use futures::StreamExt;
use lapin::{
    options::{
        BasicAckOptions, BasicQosOptions, ExchangeDeclareOptions, QueueBindOptions,
        QueueDeclareOptions,
    },
    types::{FieldTable, ShortString},
    Connection, ConnectionProperties, ExchangeKind,
};
use serde::{Deserialize, Serialize};

/// 聊天事件结构体，用于反序列化接收到的消息
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
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;

    // 声明一个名为 "notify_queue" 的持久化队列
    channel
        .queue_declare(
            ShortString::from("notify_queue"),
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;

    // 将队列绑定到交换机，使用路由键 "chat.message.created"
    // 这样只有匹配该路由键的消息才会被投递到此队列
    channel
        .queue_bind(
            ShortString::from("notify_queue"),
            ShortString::from("chat_events"),
            ShortString::from("chat.message.created"),
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;

    channel.basic_qos(10, BasicQosOptions::default()).await?; // 设置预取计数，限制未确认消息的数量

    // 开始消费 "notify_queue" 队列中的消息
    // 消费者标签为 "notify_consumer"
    let mut consumer = channel
        .basic_consume(
            ShortString::from("notify_queue"),
            ShortString::from("notify_consumer"),
            Default::default(),
            FieldTable::default(),
        )
        .await?;

    println!("Waiting for messages...");

    let mut batch: Vec<ChatEvent> = Vec::with_capacity(10); // 用于批量处理消息的缓冲区

    // 异步迭代消费消息
    while let Some(delivery_result) = consumer.next().await {
        if let Ok(delivery) = delivery_result {
            // 将消息体反序列化为 ChatEvent 结构体
            if let Ok(event) = serde_json::from_slice::<ChatEvent>(&delivery.data) {
                println!("Received message: {:?}", event);
                batch.push(event); // 将事件添加到批处理缓冲区

                // 当缓冲区达到 10 条消息时，进行批处理
                if batch.len() >= 10 {
                    println!("Processing batch of {} messages", batch.len());
                    // 在这里可以对批量消息进行处理，例如写入数据库、调用外部 API 等
                    batch.clear(); // 处理完成后清空缓冲区
                }
            } else {
                eprintln!("Failed to deserialize message: {:?}", delivery.data);
            }

            // 确认消息已被成功处理，RabbitMQ 可以将其从队列中移除
            delivery.ack(BasicAckOptions::default()).await?;
        }
    }

    Ok(())
}
