//! 速率限制代理示例
//!
//! 本示例演示如何使用 Pingora 构建一个带有速率限制功能的 HTTP 代理。
//! 限制每个 IP 地址每秒最多 3 个请求，超过限制返回 429 Too Many Requests。

use bytes::Bytes;
use once_cell::sync::Lazy;
use pingora::prelude::*;
use pingora_core::Result;
use pingora_limits::rate::Rate;
use pingora_proxy::{http_proxy_service, ProxyHttp, Session};
use std::time::Duration;

/// 全局速率限制器
///
/// 使用 `Lazy` 实现延迟初始化，确保在首次使用时才创建。
/// `Rate::new(Duration::from_secs(1))` 创建一个 1 秒时间窗口的滑动窗口限流器。
/// 该限流器基于时间衰减算法，每秒自动重置计数。
static RATE_LIMITER: Lazy<Rate> = Lazy::new(|| Rate::new(Duration::from_secs(1)));

/// 每秒最大请求数限制
const MAX_RPS: isize = 3;

/// 速率限制代理结构体
///
/// 实现 `ProxyHttp` trait 来处理 HTTP 代理逻辑，
/// 在 `request_filter` 阶段进行速率限制检查。
struct RateLimitedProxy;

#[async_trait::async_trait]
impl ProxyHttp for RateLimitedProxy {
    /// 上下文类型，用于在请求生命周期内传递数据
    /// 本示例不需要额外上下文，使用空元组
    type CTX = ();

    /// 创建新的上下文实例
    fn new_ctx(&self) -> Self::CTX {
        ()
    }

    /// 选择上游对等节点
    ///
    /// 本示例将所有请求转发到 httpbin.org，这是一个用于测试 HTTP 请求的公共服务。
    ///
    /// # 参数说明
    /// - `("httpbin.org", 80)`: 上游服务器地址和端口（HTTP 80 端口）
    /// - `false`: 不使用 TLS/HTTPS
    /// - `"httpbin.org"`: SNI（Server Name Indication），HTTP 模式下可为空
    async fn upstream_peer(
        &self,
        _session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        Ok(Box::new(HttpPeer::new(
            ("httpbin.org", 80),
            false,
            "httpbin.org".to_string(),
        )))
    }

    /// 上游请求过滤器
    ///
    /// 在请求发送到上游服务器之前修改请求头和 URI。
    /// 本示例强制将所有请求的 URI 改为 `/get`，以便从 httpbin.org 获取测试响应。
    ///
    /// # 功能
    /// - 添加 `Host` 头，确保 httpbin.org 能正确处理请求
    /// - 设置请求路径为 `/get`，获取 JSON 格式的请求信息
    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()>
    where
        Self::CTX: Send + Sync,
    {
        upstream_request.insert_header("Host", "httpbin.org")?;
        upstream_request.set_uri("/get".parse().unwrap());
        Ok(())
    }

    /// 请求过滤器 - 速率限制检查
    ///
    /// 在请求被转发到上游之前执行速率限制检查。
    /// 基于客户端 IP 地址进行限流，每个 IP 每秒最多 MAX_RPS 个请求。
    ///
    /// # 返回值
    /// - `Ok(true)`: 请求被拦截（已返回 429 错误），无需继续转发
    /// - `Ok(false)`: 请求通过检查，继续正常处理流程
    ///
    /// # 限流响应头
    /// - `X-Rate-Limit-Limit`: 每秒最大请求数
    /// - `X-Rate-Limit-Key`: 被限制的客户端标识（IP 地址）
    /// - `Retry-After`: 建议客户端在多少秒后重试
    async fn request_filter(&self, session: &mut Session, _ctx: &mut Self::CTX) -> Result<bool>
    where
        Self::CTX: Send + Sync,
    {
        // 获取客户端 IP 地址作为限流键
        // 优先使用 IPv4 地址，如果获取失败则使用 "unknown"
        let key = session
            .client_addr()
            .and_then(|addr| addr.as_inet().map(|inet| inet.ip().to_string()))
            .unwrap_or_else(|| "unknown".to_string());

        // 观察并累加该 IP 的请求计数
        // observe 方法会将当前请求计入滑动窗口，并返回当前计数
        let count = RATE_LIMITER.observe(&key, 1);

        // 检查是否超过速率限制
        if count > MAX_RPS {
            // 构建 429 Too Many Requests 响应
            let mut hdr = ResponseHeader::build(429, None)?;
            hdr.insert_header("Content-Type", "application/json")?;
            hdr.insert_header("Retry-After", "1")?; // 建议 1 秒后重试
            hdr.insert_header("X-Rate-Limit-Limit", MAX_RPS.to_string())?;
            hdr.insert_header("X-Rate-Limit-Key", key)?;

            // 发送响应头
            session.write_response_header(Box::new(hdr), false).await?;
            // 发送响应体
            session
                .write_response_body(
                    Some(Bytes::from_static(br#"{"error":"rate_limited"}"#)),
                    true,
                )
                .await?;

            // 返回 true 表示请求已被处理，无需转发到上游
            return Ok(true);
        }

        // 返回 false 表示请求通过检查，继续转发到上游
        Ok(false)
    }
}

fn main() {
    // 初始化日志系统，用于输出运行日志
    env_logger::init();

    // 解析命令行参数
    let opt = Opt::parse_args();

    // 创建 Pingora 服务器实例
    let mut server = Server::new(Some(opt)).unwrap();
    server.bootstrap();

    // 创建 HTTP 代理服务，使用 RateLimitedProxy 处理请求
    let mut proxy = http_proxy_service(&server.configuration, RateLimitedProxy);

    // 绑定 TCP 监听地址，代理将监听 0.0.0.0:6193
    proxy.add_tcp("0.0.0.0:6193");

    // 将代理服务添加到服务器
    server.add_service(proxy);

    // 启动服务器，阻塞运行直至进程终止
    server.run_forever();
}

/* 测试方法：
 *
 * 1. 启动代理服务：
 *    cargo run --example rate-limiter
 *
 * 2. 测试正常请求（前 3 个应成功）：
 *    curl http://127.0.0.1:6193/
 *
 * 3. 测试速率限制（第 4 个及以后应返回 429）：
 *    for i in {1..6}; do curl -s http://127.0.0.1:6193/; echo; done
 *
 * 预期结果：
 * - 前 3 个请求返回 httpbin.org 的 JSON 响应
 * - 第 4-6 个请求返回 {"error":"rate_limited"}
 *
 * 4. 等待 1 秒后再次请求，计数器会重置，可以继续发送 3 个请求
 */
