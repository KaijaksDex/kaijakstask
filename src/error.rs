use actix_web::{HttpResponse, ResponseError};
use derive_more::Display;
use sqlx::Error as SqlxError;
use std::io::Error as IoError;
use uuid::Error as UuidError;
use jsonwebtoken::errors::Error as JwtError;
use actix_multipart::MultipartError;

#[derive(Debug, Display)]
pub enum AppError {
    #[display(fmt = "Internal Server Error")]
    InternalServerError(String),

    #[display(fmt = "Bad Request: {}", _0)]
    BadRequest(String),

    #[display(fmt = "Unauthorized: {}", _0)]
    Unauthorized(String),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::InternalServerError(_) => {
                HttpResponse::InternalServerError().json("Internal Server Error")
            }
            AppError::BadRequest(msg) => HttpResponse::BadRequest().json(msg),
            AppError::Unauthorized(msg) => HttpResponse::Unauthorized().json(msg),
        }
    }
}

impl From<SqlxError> for AppError {
    fn from(error: SqlxError) -> AppError {
        AppError::InternalServerError(error.to_string())
    }
}

impl From<UuidError> for AppError {
    fn from(error: UuidError) -> AppError {
        AppError::BadRequest(format!("Invalid UUID: {}", error))
    }
}

impl From<IoError> for AppError {
    fn from(error: IoError) -> AppError {
        AppError::InternalServerError(format!("IO Error: {}", error))
    }
}

impl From<JwtError> for AppError {
    fn from(error: JwtError) -> AppError {
        AppError::Unauthorized(format!("JWT Error: {}", error))
    }
}

impl From<MultipartError> for AppError {
    fn from(error: MultipartError) -> AppError {
        AppError::BadRequest(format!("Multipart Error: {}", error))
    }
} 