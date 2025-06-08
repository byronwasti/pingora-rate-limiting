use crate::RateLimiter;
use async_trait::async_trait;
use pingora::{
    Result,
    http::{RequestHeader, ResponseHeader},
    proxy::{ProxyHttp, Session},
    upstreams::peer::HttpPeer,
};

pub struct RateLimitingProxy {
    upstream_addr: String,
    rate_limiter: RateLimiter,
}

impl RateLimitingProxy {
    pub fn new(upstream_addr: String, rate_limiter: RateLimiter) -> anyhow::Result<Self> {
        Ok(Self {
            upstream_addr,
            rate_limiter,
        })
    }
}

#[async_trait]
impl ProxyHttp for RateLimitingProxy {
    type CTX = ();

    fn new_ctx(&self) -> () {}

    async fn upstream_peer(&self, _session: &mut Session, _ctx: &mut ()) -> Result<Box<HttpPeer>> {
        let upstream = self.upstream_addr.clone();

        info!("upstream peer is: {:?}", upstream);

        let peer = Box::new(HttpPeer::new(
            upstream,
            false,
            "one.one.one.one".to_string(),
        ));
        Ok(peer)
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        upstream_request.insert_header("Host", "one.one.one.one")?;
        Ok(())
    }

    async fn request_filter(&self, session: &mut Session, _ctx: &mut Self::CTX) -> Result<bool>
    where
        Self::CTX: Send + Sync,
    {
        match self.rate_limiter.check_and_incr().await {
            Ok(true) => {
                let mut header = ResponseHeader::build(429, None).unwrap();
                header.insert_header("X-Rate-Limit-Limit", "0").unwrap();
                header.insert_header("X-Rate-Limit-Remaining", "1").unwrap();
                header.insert_header("X-Rate-Limit-Reset", "2").unwrap();
                session.set_keepalive(None);
                session
                    .write_response_header(Box::new(header), true)
                    .await?;
                Ok(true)
            }
            Ok(false) => Ok(false),
            Err(err) => {
                error!("Rate-limiting failed: {err}");
                Ok(false)
            }
        }
    }
}
