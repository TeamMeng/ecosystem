use async_trait::async_trait;
use pingora_core::prelude::*;
use pingora_load_balancing::{
    prelude::{RoundRobin, TcpHealthCheck},
    LoadBalancer,
};
use pingora_proxy::{http_proxy_service, ProxyHttp, Session};
use std::{sync::Arc, time::Duration};

/// 带有健康检查的负载均衡代理
///
/// 本示例演示如何创建一个带有 TCP 健康检查的负载均衡代理，
/// 自动检测后端服务器健康状态，只将请求转发给健康的后端。
struct HCLB {
    /// 负载均衡器，使用轮询（RoundRobin）策略选择后端
    upstreams: Arc<LoadBalancer<RoundRobin>>,
}

#[async_trait]
impl ProxyHttp for HCLB {
    type CTX = ();
    fn new_ctx(&self) -> Self::CTX {}

    /// 选择上游对等节点
    ///
    /// 使用轮询策略从健康后端中选择一个服务器。
    ///
    /// # 参数说明
    /// - `select(b"", 256)`: 第一个参数是哈希键（轮询时不使用），
    ///   第二个参数是最大重试次数
    async fn upstream_peer(
        &self,
        _session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        // 使用轮询策略选择后端（参数 b"" 是哈希键，轮询算法不使用它）
        let peer = self.upstreams.select(b"", 256).unwrap();

        // 创建 HTTP 对等节点
        // 注意：这里的后端是内网 IP 的 80 端口，不使用 HTTPS
        let http_peer = HttpPeer::new(peer, false, "".to_string());
        Ok(Box::new(http_peer))
    }
}

fn main() {
    env_logger::init();
    let opt = Opt::parse_args();
    let mut server = Server::new(Some(opt)).unwrap();
    server.bootstrap();

    // 1. 创建带有 3 个后端的负载均衡器
    // 注意：这些是示例内网 IP，实际使用时需要替换为真实可访问的后端地址
    let mut upstreams =
        LoadBalancer::try_from_iter(["10.0.0.1:80", "10.0.0.2:80", "10.0.0.3:80"]).unwrap();

    // 2. 配置 TCP 健康检查
    // TCP 健康检查会尝试连接后端服务器的 TCP 端口，检测是否可连通
    let hc = TcpHealthCheck::new();
    upstreams.set_health_check(hc);

    // 设置健康检查频率：每 5 秒执行一次
    // 注意：health_check_frequency 是字段不是方法
    upstreams.health_check_frequency = Some(Duration::from_secs(5));

    // 3. 启动健康检查作为后台服务
    // background_service 接收 LoadBalancer，它会定期执行健康检查
    let hc_service = background_service("health_check", upstreams);

    // 从后台服务获取 Arc<LoadBalancer> 用于代理
    let upstreams = hc_service.task();

    // 4. 使用负载均衡器创建代理服务
    let mut proxy = http_proxy_service(&server.configuration, HCLB { upstreams });
    proxy.add_tcp("0.0.0.0:6197");

    // 5. 将服务添加到服务器
    server.add_service(hc_service);
    server.add_service(proxy);

    server.run_forever();
}

/* 测试和使用说明：
 *
 * 1. 本示例使用的是示例内网 IP（10.0.0.x），这些地址默认不可访问。
 *    要测试功能，需要：
 *    - 替换为真实可访问的后端地址，或者
 *    - 在本地启动测试服务器：
 *
 *      # 启动 3 个测试后端（在不同终端）
 *      python3 -m http.server 8001 --bind 127.0.0.1
 *      python3 -m http.server 8002 --bind 127.0.0.1
 *      python3 -m http.server 8003 --bind 127.0.0.1
 *
 *    然后修改代码中的 upstreams：
 *    let mut upstreams = LoadBalancer::try_from_iter([
 *        "127.0.0.1:8001",
 *        "127.0.0.1:8002",
 *        "127.0.0.1:8003",
 *    ]).unwrap();
 *
 * 2. 测试负载均衡：
 *    for i in {1..6}; do curl http://127.0.0.1:6197/; done
 *
 * 3. 测试健康检查：
 *    - 停止其中一个后端
 *    - 等待 5 秒（健康检查周期）
 *    - 继续发送请求，会发现不再转发到已停止的后端
 */
