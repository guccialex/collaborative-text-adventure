use std::collections::HashMap;
use std::io;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use actix_cors::Cors;
use actix_web::{
    App, HttpResponse, HttpServer, Responder, get, post,
    http::header::ContentType,
    middleware, web,
};
use futures::StreamExt;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use shared::{AdventureNode, ServerMessage};

fn get_env(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

// Global counter stored in memory - persists across frontend restarts
static COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Serialize)]
struct CounterResponse {
    value: u64,
}

struct AppState {
    nodes: Vec<AdventureNode>,
    db: Connection,
}

fn seed_nodes() -> Vec<AdventureNode> {
    vec![
        AdventureNode {
            id: "cursed_mario".into(),
            parent_id: None,
            choice_text: "Super Mario 128: The Lost Cartridge".into(),
            created_by: None,
            story_text: "OK so this is a TRUE story and before you say anything, YES I know how it \
sounds, but I need to get this out there because people need to KNOW.\n\n\
It was summer 2007 and I was at my Uncle Rick's house because my mom said I \
had to \"go outside\" even though she also said I couldn't go to Tyler's house \
because Tyler's older brother had just gotten a Wii and she said I was \"obsessed.\" \
Uncle Rick's garage was full of his old stuff from when he used to work at \
(I'm not going to say the company name for legal reasons but it rhymes with \
Fintendo). I was digging through a box of cables when I found it: a cartridge, \
jet black, no label, just the word \"MARIO\" scratched into the plastic with \
something sharp. Probably a knife. Or a fingernail.\n\n\
I shouldn't have played it. I KNOW that now. But I was eleven and bored and \
Uncle Rick was asleep in his recliner with the TV on, so I grabbed his old NES \
from the shelf, blew into the cartridge (you had to do that back then, kids), \
and plugged it in.\n\n\
The title screen looked almost normal. Almost. It said \"SUPER MARIO 128\" in \
red letters, but the letters were... dripping? And the background wasn't the \
usual blue sky. It was just black. Mario was standing there on the title screen \
like usual, but he wasn't moving. He was just staring. AT ME. His eyes were \
photorealistic, which was weird because the rest of him was regular pixel Mario. \
The music was the normal theme but slowed down and reversed, and I swear — I \
SWEAR — every few seconds there was a sound like breathing.\n\n\
I pressed Start.\n\n\
World 1-1 loaded, except the ground was red and the sky was that same black. \
There were no goombas. No koopas. No coins. Just Mario, running right, in \
total silence. I played for maybe ten minutes, just running through empty \
levels that got more and more wrong — pipes that led nowhere, blocks that \
bled when you hit them, a flagpole at the end of 1-4 with something hanging \
from it that I don't want to describe.\n\n\
That's when I fell asleep. I don't remember closing my eyes. One second I was \
sitting cross-legged on Uncle Rick's carpet, and the next I was standing on \
red ground under a black sky, and my shoes were made of pixels.".into(),
        },
    ]
}

const DB_PATH: &str = "./adventure.db";

const NG_APP_ID: &str = "61527:TKbPOk1F";
const NG_GATEWAY_URL: &str = "https://newgrounds.io/gateway_v3.php";

fn init_db(path: &str) -> Connection {
    let conn = Connection::open(path).expect("Failed to open SQLite database");
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS nodes (
            id TEXT PRIMARY KEY,
            parent_id TEXT,
            choice_text TEXT NOT NULL,
            story_text TEXT NOT NULL,
            created_by TEXT,
            created_at INTEGER DEFAULT (unixepoch())
        )"
    ).expect("Failed to create nodes table");
    conn
}

fn load_nodes_from_db(conn: &Connection) -> Vec<AdventureNode> {
    let mut stmt = conn
        .prepare("SELECT id, parent_id, choice_text, story_text, created_by FROM nodes")
        .expect("Failed to prepare SELECT statement");
    let nodes = stmt
        .query_map([], |row| {
            Ok(AdventureNode {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                choice_text: row.get(2)?,
                story_text: row.get(3)?,
                created_by: row.get(4)?,
            })
        })
        .expect("Failed to query nodes")
        .filter_map(|r| r.ok())
        .collect();
    nodes
}

fn insert_node(conn: &Connection, node: &AdventureNode) {
    if let Err(e) = conn.execute(
        "INSERT OR IGNORE INTO nodes (id, parent_id, choice_text, story_text, created_by) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![node.id, node.parent_id, node.choice_text, node.story_text, node.created_by],
    ) {
        tracing::error!("Failed to insert node {}: {}", node.id, e);
    }
}

async fn verify_ng_session(client: &reqwest::Client, session_id: &str) -> Result<Option<String>, String> {
    let payload = serde_json::json!({
        "app_id": NG_APP_ID,
        "session_id": session_id,
        "call": {
            "component": "App.checkSession",
            "parameters": {}
        }
    });

    let resp = client
        .post(NG_GATEWAY_URL)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(format!("input={}", payload))
        .send()
        .await
        .map_err(|e| format!("NG gateway request failed: {}", e))?;

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("NG gateway response parse failed: {}", e))?;

    let username = body
        .pointer("/result/data/session/user/name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(username)
}

fn compute_descendant_counts(nodes: &[AdventureNode]) -> HashMap<String, u64> {
    let mut children_map: HashMap<&str, Vec<&str>> = HashMap::new();
    for node in nodes {
        if let Some(ref parent_id) = node.parent_id {
            children_map
                .entry(parent_id.as_str())
                .or_default()
                .push(&node.id);
        }
    }

    fn count(
        id: &str,
        children_map: &HashMap<&str, Vec<&str>>,
        cache: &mut HashMap<String, u64>,
    ) -> u64 {
        if let Some(&cached) = cache.get(id) {
            return cached;
        }
        let total = match children_map.get(id) {
            Some(child_ids) => child_ids
                .iter()
                .map(|cid| 1 + count(cid, children_map, cache))
                .sum(),
            None => 0,
        };
        cache.insert(id.to_string(), total);
        total
    }

    let mut cache = HashMap::new();
    for node in nodes {
        count(&node.id, &children_map, &mut cache);
    }
    cache
}

fn handle_message(msg: ServerMessage, state: &mut AppState) -> ServerMessage {
    match msg {
        ServerMessage::RequestAdventureNodes => {
            ServerMessage::ReturnAdventureNodes(state.nodes.clone())
        }
        ServerMessage::RequestDescendantCounts => {
            ServerMessage::ReturnDescendantCounts(compute_descendant_counts(&state.nodes))
        }
        ServerMessage::SubmitAdventureNode { node, .. } => {
            tracing::info!("Received new adventure node: {:?} (by {:?})", node.id, node.created_by);
            insert_node(&state.db, &node);
            state.nodes.push(node);
            ServerMessage::Ok
        }
        ServerMessage::DeleteAdventureNode { node_id, session_id: verified_username } => {
            let username = match verified_username {
                Some(u) => u,
                None => return ServerMessage::Error("Authentication required to delete nodes".into()),
            };

            let node = match state.nodes.iter().find(|n| n.id == node_id) {
                Some(n) => n,
                None => return ServerMessage::Error("Node not found".into()),
            };

            match &node.created_by {
                Some(creator) if creator == &username => {}
                _ => return ServerMessage::Error("You can only delete your own nodes".into()),
            }

            let has_children = state.nodes.iter().any(|n| n.parent_id.as_deref() == Some(&node_id));
            if has_children {
                return ServerMessage::Error("Cannot delete a node that has children".into());
            }

            tracing::info!("Deleting adventure node: {:?} (by {:?})", node_id, username);

            if let Err(e) = state.db.execute("DELETE FROM nodes WHERE id = ?1", rusqlite::params![node_id]) {
                tracing::error!("Failed to delete node {} from DB: {}", node_id, e);
                return ServerMessage::Error("Database error".into());
            }

            state.nodes.retain(|n| n.id != node_id);
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
    client: web::Data<reqwest::Client>,
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

    // If this is a SubmitAdventureNode with a session_id, verify it before locking state
    let msg = match msg {
        ServerMessage::SubmitAdventureNode { mut node, session_id } => {
            if let Some(sid) = session_id {
                match verify_ng_session(&client, &sid).await {
                    Ok(username) => {
                        node.created_by = username;
                    }
                    Err(e) => {
                        tracing::warn!("NG session verification failed: {}", e);
                        node.created_by = None;
                    }
                }
            }
            ServerMessage::SubmitAdventureNode { node, session_id: None }
        }
        ServerMessage::DeleteAdventureNode { node_id, session_id } => {
            let verified_username = if let Some(sid) = session_id {
                match verify_ng_session(&client, &sid).await {
                    Ok(username) => username,
                    Err(e) => {
                        tracing::warn!("NG session verification failed for delete: {}", e);
                        None
                    }
                }
            } else {
                None
            };
            ServerMessage::DeleteAdventureNode { node_id, session_id: verified_username }
        }
        other => other,
    };

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

#[derive(Deserialize)]
struct LlmProxyRequest {
    api_base_url: String,
    api_key: String,
    model: String,
    prompt: String,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
}

#[derive(Serialize)]
struct LlmProxyError {
    error: String,
}

#[post("/api/llm")]
async fn llm_proxy(
    body: web::Json<LlmProxyRequest>,
    client: web::Data<reqwest::Client>,
) -> HttpResponse {
    let req = body.into_inner();

    if req.api_key.is_empty() {
        return HttpResponse::BadRequest()
            .json(LlmProxyError { error: "API key is required".into() });
    }
    if req.model.is_empty() {
        return HttpResponse::BadRequest()
            .json(LlmProxyError { error: "Model name is required".into() });
    }

    let url = format!("{}/chat/completions", req.api_base_url.trim_end_matches('/'));

    let openai_body = serde_json::json!({
        "model": req.model,
        "messages": [
            { "role": "user", "content": req.prompt }
        ],
        "max_tokens": req.max_tokens.unwrap_or(1024),
        "temperature": req.temperature.unwrap_or(0.8),
        "stream": true
    });

    let response = match client
        .post(&url)
        .header("Authorization", format!("Bearer {}", req.api_key))
        .header("Content-Type", "application/json")
        .json(&openai_body)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            tracing::error!("LLM proxy request failed: {}", e);
            return HttpResponse::BadGateway()
                .json(LlmProxyError { error: format!("Request failed: {}", e) });
        }
    };

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        tracing::warn!("LLM API returned status {}: {}", status, &error_text);
        return HttpResponse::BadGateway()
            .json(LlmProxyError {
                error: format!("LLM API error ({}): {}", status, error_text),
            });
    }

    let stream = response.bytes_stream().map(|result| {
        result.map_err(|e| {
            actix_web::error::ErrorBadGateway(format!("Stream error: {}", e))
        })
    });

    HttpResponse::Ok()
        .content_type("text/event-stream")
        .insert_header(("content-encoding", "identity"))
        .insert_header(("cache-control", "no-cache"))
        .insert_header(("x-accel-buffering", "no"))
        .streaming(stream)
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

    let conn = init_db(DB_PATH);
    tracing::info!("SQLite database initialized at {}", DB_PATH);

    // Insert seed nodes (idempotent)
    for node in seed_nodes() {
        insert_node(&conn, &node);
    }

    let nodes = load_nodes_from_db(&conn);
    tracing::info!("Loaded {} adventure nodes from SQLite", nodes.len());

    let app_state = web::Data::new(Mutex::new(AppState { nodes, db: conn }));

    let http_client = web::Data::new(
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client"),
    );

    let host = get_env("HOST", "0.0.0.0");
    let port = get_env("PORT", "8080").parse::<u16>().unwrap_or(8080);

    tracing::info!("Starting HTTP server at http://{}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .app_data(http_client.clone())
            // Allow all origins
            .wrap(Cors::permissive())
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
            .service(llm_proxy)
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
