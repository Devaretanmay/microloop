mod model;

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use candle_core::Tensor;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

const HISTORY_WINDOW: usize = 5;
const SIMILARITY_THRESHOLD: f32 = 0.95;

#[derive(Deserialize)]
struct AnalyzePayload {
    session_id: String,
    tool_call: String,
    llm_error_response: String,
}

#[derive(Serialize)]
struct BlockRulePayload {
    session_id: String,
    tool_call: String,
    reason: String,
}

struct AppState {
    model: Arc<model::SemanticModel>,
    history: Mutex<HashMap<String, VecDeque<Tensor>>>,
    http_client: Client,
    proxy_url: String,
}

#[tokio::main]
async fn main() {
    println!("Starting Microloop Semantic Sidecar...");

    let semantic_model = match model::SemanticModel::new() {
        Ok(m) => Arc::new(m),
        Err(e) => {
            eprintln!("Failed to load embedding model: {}", e);
            std::process::exit(1);
        }
    };

    let proxy_url = std::env::var("PROXY_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());

    let state = Arc::new(AppState {
        model: semantic_model,
        history: Mutex::new(HashMap::new()),
        http_client: Client::new(),
        proxy_url,
    });

    let app = Router::new()
        .route("/", get(|| async { "Microloop Semantic Sidecar Running" }))
        .route("/analyze", post(handle_analyze))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8081));
    println!("Semantic sidecar listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_analyze(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AnalyzePayload>,
) -> &'static str {
    // Only analyze if there's an actual error response (failure state)
    if payload.llm_error_response.trim().is_empty() {
        return "Ignored: No error response";
    }

    let concat_text = format!(
        "Tool: {} | Error: {}",
        payload.tool_call, payload.llm_error_response
    );

    let embedding = match state.model.embed(&[&concat_text]) {
        Ok(emb) => emb,
        Err(e) => {
            eprintln!("Embedding failed: {}", e);
            return "Error";
        }
    };

    let mut history = state.history.lock().await;
    let session_history = history.entry(payload.session_id.clone()).or_insert_with(VecDeque::new);

    let mut loop_detected = false;

    // Check similarity against history
    for past_embedding in session_history.iter() {
        if let Ok(similarity) = model::cosine_similarity(&embedding, past_embedding) {
            if similarity > SIMILARITY_THRESHOLD {
                loop_detected = true;
                break;
            }
        }
    }

    session_history.push_back(embedding);
    if session_history.len() > HISTORY_WINDOW {
        session_history.pop_front();
    }

    // Release lock before making network call
    drop(history);

    if loop_detected {
        println!("Semantic loop detected for session {}", payload.session_id);
        let block_payload = BlockRulePayload {
            session_id: payload.session_id,
            tool_call: payload.tool_call,
            reason: "Semantic loop detected".to_string(),
        };

        let url = format!("{}/v1/internal/block_rule", state.proxy_url);
        let _ = state.http_client.post(&url).json(&block_payload).send().await;
    }

    "Analyzed"
}
