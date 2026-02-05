use std::{io, sync::atomic::{AtomicU64, Ordering}};

use actix_cors::Cors;
use actix_web::{
    App, HttpResponse, HttpServer, Responder, get, post,
    http::header::ContentType,
    middleware, web,
};
use serde::Serialize;

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
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Starting HTTP server at http://localhost:8080");

    HttpServer::new(move || {
        // CORS configuration to allow frontend on port 8000
        let cors = Cors::default()
            .allowed_origin("http://localhost:8000")
            .allowed_origin("http://127.0.0.1:8000")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec!["Content-Type"])
            .max_age(3600);

        App::new()
            // CORS must be registered before other middleware
            .wrap(cors)
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
    .bind(("127.0.0.1", 8080))?
    .workers(2)
    .run()
    .await
}
