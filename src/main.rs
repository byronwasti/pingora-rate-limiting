#[macro_use]
extern crate tracing;
use pingora::{
    listeners::TcpSocketOptions,
    services::{listening::Service as ListeningService},
    protocols::TcpKeepalive,
    server::{Server, configuration::Opt},
};
use std::time::Duration;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

mod proxy;
mod rate_limiter;

use proxy::RateLimitingProxy;
use rate_limiter::RateLimiter;

fn main() -> anyhow::Result<()> {
    init_tracing();

    let opt = Opt::parse_args();

    let resources = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(init_resources())?;

    let mut server = Server::new(Some(opt))?;
    server.bootstrap();

    let proxy = RateLimitingProxy::new("0.0.0.0:3000".to_string(), resources.rate_limiter)?;
    let mut proxy = pingora::proxy::http_proxy_service(&server.configuration, proxy);
    let mut options = TcpSocketOptions::default();
    options.tcp_fastopen = Some(10);
    options.tcp_keepalive = Some(TcpKeepalive {
        idle: Duration::from_secs(60),
        interval: Duration::from_secs(5),
        count: 5,
        #[cfg(target_os = "linux")]
        user_timeout: Duration::from_secs(85),
    });
    proxy.add_tcp("0.0.0.0:8000");

    let mut prometheus_service_http = ListeningService::prometheus_http_service();
    prometheus_service_http.add_tcp("127.0.0.1:6150");

    server.add_service(proxy);
    server.add_service(prometheus_service_http);
    server.run_forever();
}

struct Resources {
    rate_limiter: RateLimiter,
}

async fn init_resources() -> anyhow::Result<Resources> {
    let rate_limiter = RateLimiter::new("redis://0.0.0.0:6379").await?;

    Ok(Resources { rate_limiter })
}

fn init_tracing() {
    FmtSubscriber::builder().with_max_level(Level::INFO).init();
}
