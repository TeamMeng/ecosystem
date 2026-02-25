// use anyhow::Result;
// use async_trait::async_trait;
// use log::info;
// use pingora_core::{
//     prelude::{background_service, HttpPeer},
//     server::Server,
// };
// use pingora_http::{RequestHeader, ResponseHeader};
// use pingora_load_balancing::{prelude::TcpHealthCheck, selection::Random, LoadBalancer};
// use pingora_proxy::{ProxyHttp, Session};
// use std::{sync::Arc, time::Duration};

// // struct LB(Arc<LoadBalancer<RoundRobin>>);
// struct LB(Arc<LoadBalancer<Random>>);

// #[async_trait]
// impl ProxyHttp for LB {
//     type CTX = ();

//     fn new_ctx(&self) -> Self::CTX {}

//     async fn upstream_peer(
//         &self,
//         _session: &mut Session,
//         _ctx: &mut Self::CTX,
//     ) -> pingora_core::Result<Box<HttpPeer>> {
//         let upstream = self.0.select(b"", 256).unwrap();

//         info!("Forwarding request to {:?}", upstream);
//         Ok(Box::from(HttpPeer::new(upstream, false, String::from(""))))
//     }

//     async fn request_filter(
//         &self,
//         session: &mut Session,
//         _ctx: &mut Self::CTX,
//     ) -> pingora_core::Result<bool>
//     where
//         Self::CTX: Send + Sync,
//     {
//         if !session.req_header().uri.path().starts_with("/health") {
//             let _ = session.respond_error(403).await;
//             return Ok(true);
//         }

//         Ok(false)
//     }

//     async fn upstream_request_filter(
//         &self,
//         _session: &mut Session,
//         upstream_request: &mut RequestHeader,
//         _ctx: &mut Self::CTX,
//     ) -> pingora_core::Result<()>
//     where
//         Self::CTX: Send + Sync,
//     {
//         upstream_request
//             .insert_header("x-proxy-form", "0.0.0.0:6193")
//             .unwrap();
//         Ok(())
//     }

//     async fn response_filter(
//         &self,
//         _session: &mut Session,
//         upstream_response: &mut ResponseHeader,
//         _ctx: &mut Self::CTX,
//     ) -> pingora_core::Result<()>
//     where
//         Self::CTX: Send + Sync,
//     {
//         upstream_response.insert_header("Name", "Jack").unwrap();
//         Ok(())
//     }
// }

// fn main() -> Result<()> {
//     env_logger::init();

//     let mut server = Server::new(None)?;
//     server.bootstrap();

//     let mut upstreams = LoadBalancer::try_from_iter(["127.0.0.1:3000", "127.0.0.1:4000"])?;

//     let hc = TcpHealthCheck::new();
//     upstreams.set_health_check(hc);
//     upstreams.health_check_frequency = Some(Duration::from_secs(2));

//     let background = background_service("health checked", upstreams);
//     let upstreams = background.task();

//     let mut proxy = pingora_proxy::http_proxy_service(&server.configuration, LB(upstreams));

//     proxy.add_tcp("0.0.0.0:6193");

//     server.add_service(proxy);

//     server.run_forever();
// }

fn main() {}
