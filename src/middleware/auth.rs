use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, web
};
use futures_util::future::{ready, Ready};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

use crate::{
    config::AppConfig,
    error::AppError,
    models::Claims,
};

pub struct JwtAuth;

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = JwtAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct JwtAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let config = req.app_data::<web::Data<AppConfig>>().unwrap().clone();

        Box::pin(async move {
            if let Some(auth_header) = req.headers().get("Authorization") {
                if let Ok(auth_str) = auth_header.to_str() {
                    if auth_str.starts_with("Bearer ") {
                        let token = auth_str[7..].to_string();
                        
                        let mut validation = Validation::default();
                        validation.validate_exp = false;
                        
                        match decode::<Claims>(
                            &token,
                            &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
                            &validation,
                        ) {
                            Ok(token_data) => {
                                let claims = token_data.claims;
                                req.extensions_mut().insert(claims);
                                service.call(req).await
                            }
                            Err(e) => {
                                Err(AppError::Unauthorized(format!("Invalid token: {}", e)).into())
                            }
                        }
                    } else {
                        Err(AppError::Unauthorized("Invalid token format".to_string()).into())
                    }
                } else {
                    Err(AppError::Unauthorized("Invalid authorization header".to_string()).into())
                }
            } else {
                Err(AppError::Unauthorized("Missing authorization header".to_string()).into())
            }
        })
    }
}

pub struct RequireAuth;

impl<S, B> Transform<S, ServiceRequest> for RequireAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RequireAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequireAuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct RequireAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RequireAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            if req.extensions().get::<Claims>().is_some() {
                service.call(req).await
            } else {
                Err(AppError::Unauthorized("Authentication required".to_string()).into())
            }
        })
    }
} 