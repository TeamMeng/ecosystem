use async_trait::async_trait;
// HttpPeer: 表示上游服务器的连接信息（地址、端口、TLS、SNI等）
use pingora::prelude::HttpPeer;
// Opt, Server: 命令行参数解析和服务器核心
use pingora_core::prelude::*;
// Result: Pingora 的错误处理类型
use pingora_core::Result;
// RequestHeader: HTTP 请求头，用于修改 upstream 请求
use pingora_http::RequestHeader;
// http_proxy_service: 创建 HTTP 代理服务的工厂函数
use pingora_proxy::http_proxy_service;
// ProxyHttp trait: 实现代理逻辑的核心 trait, Session: 表示一个客户端连接会话
use pingora_proxy::{ProxyHttp, Session};

// 定义我们的代理结构体
pub struct MinimalProxy;

// async_trait: 允许在 trait 中使用 async fn
#[async_trait]
impl ProxyHttp for MinimalProxy {
    // CTX (Context): 用于在同一次请求的不同阶段传递数据
    // 这里用 () 表示不需要传递额外数据
    type CTX = ();

    // 创建一个新的上下文实例，每个请求都会调用
    fn new_ctx(&self) -> Self::CTX {}

    /// 选择上游服务器（核心方法）
    /// 当客户端请求到来时，这个方法决定把请求转发到哪个上游服务器
    ///
    /// 参数:
    ///   - _session: 客户端会话，包含请求信息（可用来读取 Host、Path 等）
    ///   - _ctx: 上下文，用于在同一次请求的不同回调间传递数据
    ///
    /// 返回:
    ///   - Result<Box<HttpPeer>>: 上游服务器的连接信息
    async fn upstream_peer(
        &self,
        _session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        // 创建上游服务器配置：
        // - ("httpbin.org", 443): 上游地址和端口
        // - true: 启用 TLS/HTTPS
        // - "httpbin.org".into(): SNI (Server Name Indication)，TLS 握手时告诉服务器要访问的域名
        let peer = HttpPeer::new(("httpbin.org", 80), false, "".into());

        Ok(Box::new(peer))
    }

    /// 修改发往上游的请求（请求过滤器）
    /// 在把请求转发给上游服务器之前，可以修改请求头、URI 等
    ///
    /// 参数:
    ///   - _session: 客户端会话
    ///   - upstream_request: 将要发送给上游的 HTTP 请求（可修改）
    ///   - _ctx: 上下文
    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()>
    where
        Self::CTX: Send + Sync,
    {
        // 设置 Host 头，告诉上游服务器我们要访问哪个虚拟主机
        // 这对支持多域名的服务器（如 CDN）非常重要
        upstream_request.insert_header("Host", "httpbin.org")?;

        // 修改请求路径：把所有请求都重定向到 /get
        // 这意味着无论访问什么路径，上游只会看到 /get
        upstream_request.set_uri("/get".parse().unwrap());
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    // 初始化日志系统，从环境变量 RUST_LOG 读取日志级别
    // 例如：RUST_LOG=info cargo run
    env_logger::init();

    // 解析命令行参数（--help 查看可用选项）
    // 默认支持：-c 配置文件, -d 守护进程, -t 线程数等
    let opt = Opt::parse_args();

    // 创建服务器实例
    let mut server = Server::new(Some(opt))?;

    // 启动引导服务（加载配置、初始化组件等）
    server.bootstrap();

    // 创建 HTTP 代理服务，绑定我们的 MinimalProxy 实现
    let mut proxy = http_proxy_service(&server.configuration, MinimalProxy);

    // 监听 TCP 端口 0.0.0.0:6188
    // 0.0.0.0 表示监听所有网络接口
    proxy.add_tcp("0.0.0.0:6188");

    // 将代理服务添加到服务器
    server.add_service(proxy);

    // 启动事件循环，阻塞运行直到收到终止信号（如 Ctrl+C）
    server.run_forever();
}
