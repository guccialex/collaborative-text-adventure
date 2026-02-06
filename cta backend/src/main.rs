use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use actix_cors::Cors;
use actix_web::{
    App, HttpResponse, HttpServer, Responder, get, post,
    http::header::ContentType,
    middleware, web,
};
use serde::Serialize;
use shared::{AdventureNode, ServerMessage};

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

struct AppState {
    nodes: Vec<AdventureNode>,
}

fn seed_nodes() -> Vec<AdventureNode> {
    vec![
        AdventureNode {
            id: "root".into(),
            parent_id: None,
            choice_text: "Root".into(),
            story_text: "This is a branching text adventure. Click on one of the options to read it and see its branching paths, or contribute one of your own at any point.".into(),
        },
        AdventureNode {
            id: "torch_passage".into(),
            parent_id: Some("root".into()),
            choice_text: "Take the torch and explore the dark passage ahead".into(),
            story_text: "You grab the torch from its sconce. The warmth is comforting against the chill. The passage ahead slopes downward, and you can hear the faint sound of dripping water echoing from somewhere deeper within.".into(),
        },
        AdventureNode {
            id: "search_chamber".into(),
            parent_id: Some("root".into()),
            choice_text: "Search the chamber for clues about your identity".into(),
            story_text: "You run your hands along the rough stone walls, searching for anything that might explain your situation. In a corner, your fingers brush against something metallic - a small iron key, covered in cobwebs. Near it lies a torn piece of parchment with faded writing.".into(),
        },
        AdventureNode {
            id: "call_out".into(),
            parent_id: Some("root".into()),
            choice_text: "Call out into the darkness".into(),
            story_text: "\"Hello?\" your voice echoes through the chamber, bouncing off unseen walls in the darkness. For a moment, silence. Then... footsteps. Slow, deliberate footsteps approaching from the passage ahead. A raspy voice calls back: \"Another one awakens...\"".into(),
        },
        AdventureNode {
            id: "deep_passage".into(),
            parent_id: Some("torch_passage".into()),
            choice_text: "Continue deeper into the passage".into(),
            story_text: "The passage opens into a vast underground cavern. Your torchlight barely reaches the ceiling high above. In the center, an ancient stone altar stands, covered in strange symbols that seem to glow faintly. Three corridors branch off in different directions.".into(),
        },
    ]
}

const NODES_FILE: &str = "./adventurenodes.jsonl";

fn load_nodes_from_file() -> Vec<AdventureNode> {
    let path = Path::new(NODES_FILE);
    if !path.exists() {
        return Vec::new();
    }
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            tracing::error!("Failed to open {}: {}", NODES_FILE, e);
            return Vec::new();
        }
    };
    let reader = io::BufReader::new(file);
    let mut nodes = Vec::new();
    for (i, line) in reader.lines().enumerate() {
        match line {
            Ok(line) if line.trim().is_empty() => continue,
            Ok(line) => match serde_json::from_str::<AdventureNode>(&line) {
                Ok(node) => nodes.push(node),
                Err(e) => tracing::warn!("Skipping invalid line {} in {}: {}", i + 1, NODES_FILE, e),
            },
            Err(e) => tracing::warn!("Failed to read line {} in {}: {}", i + 1, NODES_FILE, e),
        }
    }
    nodes
}

fn append_node_to_file(node: &AdventureNode) {
    let mut file = match OpenOptions::new().create(true).append(true).open(NODES_FILE) {
        Ok(f) => f,
        Err(e) => {
            tracing::error!("Failed to open {} for writing: {}", NODES_FILE, e);
            return;
        }
    };
    match serde_json::to_string(node) {
        Ok(json) => {
            if let Err(e) = writeln!(file, "{}", json) {
                tracing::error!("Failed to write to {}: {}", NODES_FILE, e);
            }
        }
        Err(e) => tracing::error!("Failed to serialize node to JSON: {}", e),
    }
}

fn handle_message(msg: ServerMessage, state: &mut AppState) -> ServerMessage {
    match msg {
        ServerMessage::RequestAdventureNodes => {
            ServerMessage::ReturnAdventureNodes(state.nodes.clone())
        }
        ServerMessage::SubmitAdventureNode(node) => {
            tracing::info!("Received new adventure node: {:?}", node.id);
            append_node_to_file(&node);
            state.nodes.push(node);
            ServerMessage::Ok
        }
        other => {
            tracing::warn!("Unhandled message type: {:?}", std::mem::discriminant(&other));
            ServerMessage::Error("Unhandled message type".into())
        }
    }
}

#[post("/api")]
async fn api_bincode(
    body: web::Bytes,
    data: web::Data<Mutex<AppState>>,
) -> impl Responder {
    let msg = match bincode::deserialize::<ServerMessage>(&body) {
        Ok(msg) => msg,
        Err(e) => {
            tracing::error!("Bincode deserialization error: {:?}", e);
            return HttpResponse::BadRequest()
                .body(format!("Failed to deserialize: {}", e));
        }
    };

    tracing::info!("Received API message: {:?}", std::mem::discriminant(&msg));

    let response = {
        let mut state = data.lock().unwrap();
        handle_message(msg, &mut state)
    };

    match bincode::serialize(&response) {
        Ok(bytes) => HttpResponse::Ok()
            .content_type("application/octet-stream")
            .body(bytes),
        Err(e) => {
            tracing::error!("Bincode serialization error: {:?}", e);
            HttpResponse::InternalServerError()
                .body(format!("Failed to serialize: {}", e))
        }
    }
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

    let mut nodes = seed_nodes();
    tracing::info!("Loaded {} seed adventure nodes", nodes.len());

    let file_nodes = load_nodes_from_file();
    tracing::info!("Loaded {} adventure nodes from {}", file_nodes.len(), NODES_FILE);
    nodes.extend(file_nodes);

    let app_state = web::Data::new(Mutex::new(AppState { nodes }));

    let host = get_env("HOST", "0.0.0.0");
    let port = get_env("PORT", "8080").parse::<u16>().unwrap_or(8080);

    tracing::info!("Starting HTTP server at http://{}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            // Allow all origins
            .wrap(cors_permissive())
            // enable automatic response compression
            .wrap(middleware::Compress::default())
            // enable logger
            .wrap(middleware::Logger::default().log_target("@"))
            // routes
            .service(index)
            .service(health)
            .service(api_bincode)
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
