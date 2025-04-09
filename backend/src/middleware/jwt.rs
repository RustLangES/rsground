use crate::auth::handlers::Claims;
use actix_web::{
    body::EitherBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures::future::{ok, LocalBoxFuture, Ready};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::env;

pub struct JwtMiddleware;

impl<S, B> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = JwtMiddlewareMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(JwtMiddlewareMiddleware { service })
    }
}

pub struct JwtMiddlewareMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for JwtMiddlewareMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if let Some(auth_header) = req.headers().get("Authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = auth_str.trim_start_matches("Bearer ").to_string();
                    let secret_key = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".into());
                    let validation = Validation::default();
                    if decode::<Claims>(
                        &token,
                        &DecodingKey::from_secret(secret_key.as_ref()),
                        &validation,
                    )
                    .is_ok()
                    {
                        let fut = self.service.call(req);
                        return Box::pin(async move {
                            let res = fut.await?;
                            return Ok(res.map_into_left_body());
                        });
                    }
                }
            }
        }
        Box::pin(async {
            let res = HttpResponse::Unauthorized().finish().map_into_right_body();
            Ok(req.into_response(res))
        })
    }
}
