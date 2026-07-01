mod model;

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use candle_core::Tensor;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

const HISTORY_WINDOW: usize = 5;
const SIMILARITY_THRESHOLD: f32 = 0.85;

#[derive(Deserialize)]
struct AnalyzePayload {
    session_id: String,
    tool_call: String,
    llm_error_response: String,
}

#[derive(Serialize)]
struct AnalyzeResponse {
    loop_detected: bool,
}

struct AppState {
    model: Arc<model::SemanticModel>,
    history: Mutex<HashMap<String, VecDeque<Tensor>>>,
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

    let state = Arc::new(AppState {
        model: semantic_model,
        history: Mutex::new(HashMap::new()),
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
) -> Json<AnalyzeResponse> {
    let concat_text = format!(
        "Tool: {} | Error: {}",
        payload.tool_call, payload.llm_error_response
    );

    let embedding = match state.model.embed(&[&concat_text]) {
        Ok(emb) => emb,
        Err(e) => {
            eprintln!("Embedding failed: {}", e);
            return Json(AnalyzeResponse { loop_detected: false });
        }
    };

    let mut history = state.history.lock().await;
    let session_history = history.entry(payload.session_id.clone()).or_insert_with(VecDeque::new);

    let mut loop_detected = false;

    for past_embedding in session_history.iter() {
        if model::cosine_similarity(&embedding, past_embedding)
            .map(|s| s > SIMILARITY_THRESHOLD)
            .unwrap_or(false)
        {
            loop_detected = true;
            break;
        }
    }

    session_history.push_back(embedding);
    if session_history.len() > HISTORY_WINDOW {
        session_history.pop_front();
    }

    if loop_detected {
        println!("Semantic loop detected: session={} tool={}", payload.session_id, payload.tool_call);
    }

    Json(AnalyzeResponse { loop_detected })
}
