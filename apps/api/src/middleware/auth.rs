use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    http::header,
    web, Error, HttpMessage,
};
use futures::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::task::{Context, Poll};

use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

use crate::config::Config;
use application::auth::dtos::Claims;

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService { service }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
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
        // Try to extract and validate JWT if Authorization header is present.
        // If header is missing, we let the request pass through.
        // If header is present but invalid, we return 401.
        if let Some(auth_header_value) = req.headers().get(header::AUTHORIZATION) {
            if let Ok(auth_str) = auth_header_value.to_str() {
                if let Some(token) = auth_str
                    .strip_prefix("Bearer ")
                    .or_else(|| auth_str.strip_prefix("bearer "))
                {
                    if let Some(config) = req.app_data::<web::Data<Config>>() {
                        let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_bytes());
                        let mut validation = Validation::new(Algorithm::HS256);
                        validation.validate_exp = true;

                        match decode::<Claims>(token, &decoding_key, &validation) {
                            Ok(token_data) => {
                                // Put Claims into request extensions so handlers can read them.
                                req.extensions_mut().insert(token_data.claims);
                            }
                            Err(_) => {
                                return Box::pin(async move {
                                    Err(ErrorUnauthorized("Invalid or expired token"))
                                });
                            }
                        }
                    }
                }
            }
        }

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}
