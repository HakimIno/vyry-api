use actix_web::{FromRequest, HttpMessage};
use application::auth::dtos::Claims;
use futures::future::{ready, Ready};
use actix_web::Error;

pub struct AuthUser(pub Claims);

impl FromRequest for AuthUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        match req.extensions().get::<Claims>() {
            Some(claims) => ready(Ok(AuthUser(claims.clone()))), // Cloning claims is cheap? It has Strings. 
            // Better if Claims was Copy, but it's not. 
            // We can change AuthUser to hold reference? No, FromRequest requires 'static or owned usually.
            // Let's assume Clone is fine.
            None => ready(Err(actix_web::error::ErrorUnauthorized("Unauthorized"))),
        }
    }
}

impl std::ops::Deref for AuthUser {
    type Target = Claims;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
