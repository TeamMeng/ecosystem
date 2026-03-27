use async_trait::async_trait;
use pingora::{
    prelude::{HttpPeer, Opt},
    server::Server,
};
use pingora_core::Result;
use pingora_http::{RequestHeader, ResponseHeader};
use pingora_proxy::{http_proxy_service, ProxyHttp, Session};

/// 管理员接口的认证令牌
/// 实际生产环境中应从环境变量或配置中心读取
const AUTH_TOKEN: &str = "my-secret-token";

/// API 网关结构体
/// 实现了 `ProxyHttp` trait 来处理 HTTP 代理逻辑
pub struct ApiGateway;

#[async_trait]
impl ProxyHttp for ApiGateway {
    /// 上下文类型，用于在请求生命周期内传递数据
    /// 此处不需要额外上下文，使用空元组
    type CTX = ();

    /// 创建新的上下文实例
    fn new_ctx(&self) -> Self::CTX {}

    /// 请求过滤器 - 在请求被转发到上游之前执行
    /// 用于身份验证、速率限制等前置处理
    ///
    /// 返回值说明:
    /// - `Ok(true)`: 请求已被处理（如直接返回错误响应），无需继续转发
    /// - `Ok(false)`: 请求通过过滤，继续正常处理流程
    async fn request_filter(&self, session: &mut Session, _ctx: &mut Self::CTX) -> Result<bool>
    where
        Self::CTX: Send + Sync,
    {
        let path = session.req_header().uri.path();

        // 对 /admin 路径进行身份验证保护
        if path.starts_with("/admin") {
            // 从请求头中提取 Authorization 头
            let auth = session
                .req_header()
                .headers
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .map(|v| v.strip_prefix("Bearer ").unwrap_or(""));

            match auth {
                // 令牌验证通过，继续处理
                Some(t) if t == AUTH_TOKEN => return Ok(false),
                // 验证失败，返回 403 Forbidden
                _ => {
                    let mut hdr = ResponseHeader::build(403, None)?;
                    hdr.insert_header("Content-Type", "text/plain").unwrap();
                    session.write_response_header(Box::new(hdr), false).await?;
                    session
                        .write_response_body(Some("Forbidden".into()), true)
                        .await?;
                    return Ok(true);
                }
            }
        }

        // 非受保护路径，直接放行
        Ok(false)
    }

    /// 上游请求过滤器 - 在请求发送到上游服务器之前修改请求
    /// 可用于路径重写、添加头部等
    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()>
    where
        Self::CTX: Send + Sync,
    {
        let path = upstream_request.uri.path();

        // 路径重写：移除 /api 前缀
        // 例如: /api/users/1 -> /users/1
        if let Some(real_path) = path.strip_prefix("/api") {
            upstream_request.set_uri(real_path.parse().unwrap());
        }

        Ok(())
    }

    /// 响应过滤器 - 在响应返回给客户端之前修改响应头
    /// 可用于添加安全头、隐藏服务器信息等
    async fn response_filter(
        &self,
        _session: &mut Session,
        upstream_response: &mut ResponseHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()>
    where
        Self::CTX: Send + Sync,
    {
        // 添加自定义响应头，标识网关信息
        upstream_response
            .insert_header("X-Gateway", "Pingora/Demo")
            .unwrap();
        upstream_response
            .insert_header("X-Powered-By", "Rust")
            .unwrap();

        // 移除 Server 头，隐藏上游服务器信息（安全考虑）
        upstream_response.remove_header("Server");

        Ok(())
    }

    /// 上游对等节点选择 - 决定请求转发到哪个上游服务器
    /// 可在此实现负载均衡、服务发现等逻辑
    async fn upstream_peer(
        &self,
        _session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        // 创建上游 HTTP 对等节点
        // 参数: (地址, 是否使用 HTTPS, SNI 主机名)
        let peer = HttpPeer::new(("httpbin", 80), false, "".to_string());
        Ok(Box::new(peer))
    }
}

fn main() {
    // 初始化日志系统
    env_logger::init();

    // 解析命令行参数
    let opt = Opt::parse_args();

    // 创建 Pingora 服务器实例
    let mut server = Server::new(Some(opt)).unwrap();
    server.bootstrap();

    // 创建 HTTP 代理服务
    let mut gw = http_proxy_service(&server.configuration, ApiGateway);

    // 绑定 TCP 监听地址
    gw.add_tcp("0.0.0.0:6190");

    // 将服务添加到服务器
    server.add_service(gw);

    // 启动服务器，阻塞运行
    server.run_forever();
}
