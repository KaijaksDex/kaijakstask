use actix_multipart::Multipart;
use actix_web::{web, HttpResponse, HttpRequest, post, HttpMessage};
use futures_util::StreamExt;
use mime_guess;
use sqlx::PgPool;
use std::io::Write;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{Claims, UploadResponse},
};

#[post("/upload")]
pub async fn upload_image(
    req: HttpRequest,
    _pool: web::Data<PgPool>,
    mut payload: Multipart,
) -> Result<HttpResponse, AppError> {
    let extensions = req.extensions();
    let claims = extensions.get::<Claims>()
        .ok_or_else(|| AppError::Unauthorized("Missing authentication".to_string()))?;
    
    let _user_id = Uuid::parse_str(&claims.sub)?;
    let mut image_data = Vec::new();
    let mut filename = String::from("unknown.jpg");
    let mut has_image = false;

    while let Some(field) = payload.next().await {
        let mut field = field?;
        let content_disp = field.content_disposition();
        let field_name = content_disp.get_name().ok_or_else(|| {
            AppError::BadRequest("Field name not found".to_string())
        })?;
        
        if field_name == "image" {
            has_image = true;
            if let Some(name) = content_disp.get_filename() {
                filename = name.to_string();
            }

            while let Some(chunk) = field.next().await {
                let data = chunk?;
                image_data.extend_from_slice(&data);
            }
        } else {
            while let Some(chunk) = field.next().await {
                let _ = chunk?;
            }
        }
    }
    
    if !has_image || image_data.is_empty() {
        return Err(AppError::BadRequest("No image found in request".to_string()));
    }

    let mime_type = mime_guess::from_path(&filename)
        .first_or_octet_stream();
    
    if !mime_type.type_().as_str().eq("image") {
        return Err(AppError::BadRequest("File must be an image".to_string()));
    }

    let file_extension = filename.split('.').last().unwrap_or("jpg");
    let unique_filename = format!("image_{}.{}", Uuid::new_v4(), file_extension);
    let file_path = format!("uploads/{}", unique_filename);

    let mut file = std::fs::File::create(&file_path)?;
    file.write_all(&image_data)?;

    Ok(HttpResponse::Ok().json(UploadResponse {
        image_url: format!("/uploads/{}", unique_filename),
    }))
} 