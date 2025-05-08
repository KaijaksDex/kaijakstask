use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};
use std::string::FromUtf8Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Todo {
    pub id: Uuid,
    pub user_id: Uuid,
    pub text: String,
    pub date: NaiveDate,
    pub image_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CreateTodoRequest {
    pub text: String,
    pub date: NaiveDate,
}

#[derive(Debug, Serialize)]
pub struct CreateTodoResponse {
    pub id: Uuid,
    pub text: String,
    pub date: NaiveDate,
    pub image_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub image_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

impl From<FromUtf8Error> for crate::error::AppError {
    fn from(error: FromUtf8Error) -> Self {
        crate::error::AppError::BadRequest(format!("Invalid UTF-8 data: {}", error))
    }
} 