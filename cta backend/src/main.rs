use std::{io, sync::atomic::{AtomicU64, Ordering}};

use actix_cors::Cors;
use actix_web::{
    App, HttpResponse, HttpServer, Responder, get, post,
    http::header::ContentType,
    middleware, web,
};
use serde::Serialize;

fn get_env(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn cors_permissive() -> Cors {
    Cors::permissive()
}

// Global counter stored in memory - persists across frontend restarts
static COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Serialize)]
struct CounterResponse {
    value: u64,
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .body("Welcome to the Collaborative Text Adventure API!")
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(r#"{"status": "ok"}"#)
}

#[get("/api/counter")]
async fn get_counter() -> impl Responder {
    let value = COUNTER.load(Ordering::SeqCst);
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .json(CounterResponse { value })
}

#[post("/api/counter/increment")]
async fn increment_counter() -> impl Responder {
    let value = COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
    tracing::info!("Counter incremented to {}", value);
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .json(CounterResponse { value })
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    dotenv::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let host = get_env("HOST", "0.0.0.0");
    let port = get_env("PORT", "8080").parse::<u16>().unwrap_or(8080);

    tracing::info!("Starting HTTP server at http://{}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            // Allow all origins
            .wrap(cors_permissive())
            // enable automatic response compression
            .wrap(middleware::Compress::default())
            // enable logger
            .wrap(middleware::Logger::default().log_target("@"))
            // routes
            .service(index)
            .service(health)
            .service(get_counter)
            .service(increment_counter)
            // default 404
            .default_service(web::to(|| async {
                HttpResponse::NotFound().body("Not Found")
            }))
    })
    .bind((host.as_str(), port))?
    .workers(2)
    .run()
    .await
}
