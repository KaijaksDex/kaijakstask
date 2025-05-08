use actix_web::{web, HttpResponse, HttpRequest, post, get, HttpMessage};
use sqlx::{PgPool, query};
use uuid::Uuid;
use chrono::{NaiveDate, Utc};
use actix_multipart::Multipart;
use futures_util::StreamExt;
use std::io::Write;

use crate::{
    error::AppError,
    models::{Claims, CreateTodoResponse, Todo},
};

#[post("/todos")]
pub async fn create_todo(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    mut payload: Multipart,
) -> Result<HttpResponse, AppError> {
    let extensions = req.extensions();
    let claims = extensions.get::<Claims>()
        .ok_or_else(|| AppError::Unauthorized("Missing authentication".to_string()))?;
    
    let user_id = Uuid::parse_str(&claims.sub)?;
    
    let mut text = None;
    let mut date_str = None;
    let mut image_data = Vec::new();
    let mut filename = String::new();
    let mut has_image = false;
    
    while let Some(field) = payload.next().await {
        let mut field = field?;
        let content_disposition = field.content_disposition();
        let field_name = content_disposition.get_name().ok_or_else(|| {
            AppError::BadRequest("Field name not found".to_string())
        })?;
        
        match field_name {
            "text" => {
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    data.extend_from_slice(&chunk?);
                }
                text = Some(String::from_utf8(data)?);
            },
            "date" => {
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    data.extend_from_slice(&chunk?);
                }
                date_str = Some(String::from_utf8(data)?);
            },
            "image" => {
                has_image = true;
                
                if let Some(name) = content_disposition.get_filename() {
                    filename = name.to_string();
                }
                
                while let Some(chunk) = field.next().await {
                    let data = chunk?;
                    image_data.extend_from_slice(&data);
                }
            },
            _ => {
                while let Some(chunk) = field.next().await {
                    let _ = chunk?;
                }
            }
        }
    }
    
    let text = text.ok_or_else(|| AppError::BadRequest("Missing text field".to_string()))?;
    let date_str = date_str.ok_or_else(|| AppError::BadRequest("Missing date field".to_string()))?;
    
    let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest("Invalid date format, use YYYY-MM-DD".to_string()))?;
    
    let mut image_url = None;
    if has_image && !image_data.is_empty() {
        let mime_type = mime_guess::from_path(&filename)
            .first_or_octet_stream();
        
        if !mime_type.type_().as_str().eq("image") {
            return Err(AppError::BadRequest("File must be an image".to_string()));
        }
        
        let file_extension = filename.split('.').last().unwrap_or("jpg");
        let todo_id = Uuid::new_v4();
        let unique_filename = format!("todo_{}.{}", todo_id, file_extension);
        let file_path = format!("uploads/{}", unique_filename);
        
        let mut file = std::fs::File::create(&file_path)?;
        file.write_all(&image_data)?;
        
        image_url = Some(format!("/uploads/{}", unique_filename));
    }
    
    let row = query!(
        r#"
        INSERT INTO todos (user_id, text, date, image_url)
        VALUES ($1, $2, $3, $4)
        RETURNING id, text, date, image_url
        "#,
        user_id,
        text,
        date,
        image_url
    )
    .fetch_one(&**pool)
    .await?;
    
    Ok(HttpResponse::Created().json(CreateTodoResponse {
        id: row.id,
        text: row.text,
        date: row.date,
        image_url: row.image_url,
    }))
}

#[get("/todos")]
pub async fn get_todos(
    req: HttpRequest,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let extensions = req.extensions();
    let claims = extensions.get::<Claims>()
        .ok_or_else(|| AppError::Unauthorized("Missing authentication".to_string()))?;
    
    let user_id = Uuid::parse_str(&claims.sub)?;

    let rows = query!(
        r#"
        SELECT id, user_id, text, date, image_url, created_at
        FROM todos
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
        user_id
    )
    .fetch_all(&**pool)
    .await?;

    let todos: Vec<Todo> = rows
        .into_iter()
        .map(|row| Todo {
            id: row.id,
            user_id: row.user_id,
            text: row.text,
            date: row.date,
            image_url: row.image_url,
            created_at: row.created_at.unwrap_or_else(|| Utc::now()),
        })
        .collect();

    Ok(HttpResponse::Ok().json(todos))
} 