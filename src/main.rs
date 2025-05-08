mod config;
mod models;
mod handlers;
mod middleware;
mod error;

use actix_web::{web, App, HttpServer};
use actix_files as files;
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let server_url = format!("{}:{}", host, port);

    std::fs::create_dir_all("uploads").expect("Failed to create uploads directory");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    println!("Server running at http://{}", server_url);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config::AppConfig::from_env()))
            .service(handlers::auth::login)
            .service(
                files::Files::new("/uploads", "./uploads")
                    .use_last_modified(true)
            )
            .service(
                web::scope("")
                    .wrap(middleware::auth::RequireAuth)
                    .wrap(middleware::auth::JwtAuth)
                    .service(handlers::todos::create_todo)
                    .service(handlers::todos::get_todos)
                    .service(handlers::upload::upload_image)
            )
    })
    .bind(server_url)?
    .run()
    .await
}
