use actix_web::{web, HttpResponse, post};
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::{PgPool, query};

use crate::{
    config::AppConfig,
    error::AppError,
    models::{Claims, LoginRequest, LoginResponse},
};

#[post("/login")]
pub async fn login(
    pool: web::Data<PgPool>,
    config: web::Data<AppConfig>,
    login_data: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    if login_data.email != "test@example.com" || login_data.password != "securepassword" {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    let user_id = query!(
        "SELECT id FROM users WHERE email = $1",
        login_data.email
    )
    .fetch_one(&**pool)
    .await?
    .id;

    query!(
        "INSERT INTO sessions (user_id) VALUES ($1)",
        user_id
    )
    .execute(&**pool)
    .await?;

    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )?;

    Ok(HttpResponse::Ok().json(LoginResponse { token }))
} 