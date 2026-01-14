use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::future::LocalBoxFuture;
use governor::{
    clock::DefaultClock,
    state::keyed::DashMapStateStore,
    Quota, RateLimiter,
};
use std::future::{ready, Ready};
use std::rc::Rc;
use std::task::{Context, Poll};
use std::num::NonZeroU32;
use tracing::warn;

/// Per-IP rate limiting middleware
pub struct PerIpRateLimitMiddleware {
    limiter: Rc<RateLimiter<String, DashMapStateStore<String>, DefaultClock>>,
}

impl PerIpRateLimitMiddleware {
    pub fn new(requests_per_minute: u32) -> Self {
        let quota = Quota::per_minute(nonzero_or_one(requests_per_minute));
        let limiter = RateLimiter::keyed(quota);
        Self {
            limiter: Rc::new(limiter),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for PerIpRateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = PerIpRateLimitService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(PerIpRateLimitService {
            service,
            limiter: self.limiter.clone(),
        }))
    }
}

pub struct PerIpRateLimitService<S> {
    service: S,
    limiter: Rc<RateLimiter<String, DashMapStateStore<String>, DefaultClock>>,
}

impl<S, B> Service<ServiceRequest> for PerIpRateLimitService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Extract IP address
        let ip = req
            .connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown")
            .to_string();

        // Check rate limit per IP
        if self.limiter.check_key(&ip).is_err() {
            warn!("Rate limit exceeded for IP: {}", ip);
            return Box::pin(async move {
                Err(actix_web::error::ErrorTooManyRequests(
                    "Rate limit exceeded. Please try again later.",
                ))
            });
        }

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}

#[inline]
fn nonzero_or_one(val: u32) -> NonZeroU32 {
    NonZeroU32::new(val).unwrap_or_else(|| NonZeroU32::new(1).unwrap())
}
