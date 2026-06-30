use axum::{
    Router,
    routing::{get, post},
};
use std::env;
use std::net::SocketAddr;

mod proxy;

#[tokio::main]
async fn main() {
    println!("Starting Microloop Proxy...");

    let yaml = match std::fs::read_to_string("microloop.yaml") {
        Ok(y) => y,
        Err(_) => {
            eprintln!("Failed to read microloop.yaml");
            std::process::exit(1);
        }
    };

    let microloop_state = microloop::state::MicroloopState::new(&yaml).unwrap_or_else(|e| {
        eprintln!("Failed to initialize Microloop core: {}", e);
        std::process::exit(1);
    });
    println!("Microloop core initialized.");

    let (sidecar_tx, mut sidecar_rx) = tokio::sync::mpsc::channel::<proxy::AnalyzePayload>(1000);

    let state = proxy::AppState {
        microloop_state: std::sync::Arc::new(std::sync::Mutex::new(microloop_state)),
        semantic_blocklist: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        target_base_url: env::var("TARGET_API_URL")
            .unwrap_or_else(|_| "https://api.openai.com".to_string()),
        api_key: env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
            eprintln!("Warning: OPENAI_API_KEY not set, upstream requests will likely 401");
            String::new()
        }),
        sidecar_tx,
    };

    let app = Router::new()
        .route("/v1/chat/completions", post(proxy::handle_proxy_request))
        .route("/v1/messages", post(proxy::handle_proxy_request))
        .route("/v1/internal/block_rule", post(proxy::handle_block_rule))
        .route("/", get(|| async { "Microloop Proxy Running" }))
        .with_state(state.clone());

    // Background task to send payloads to sidecar
    tokio::spawn(async move {
        let client = reqwest::Client::new();
        let sidecar_url = std::env::var("SIDECAR_URL").unwrap_or_else(|_| "http://127.0.0.1:8081".to_string());
        
        while let Some(payload) = sidecar_rx.recv().await {
            let _ = client.post(format!("{}/analyze", sidecar_url))
                .json(&payload)
                .send()
                .await;
        }
    });

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Microloop proxy listening on {}", addr);

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Fatal: Could not bind to {}: {}", addr, e);
            std::process::exit(1);
        }
    };
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("Fatal: Server error: {}", e);
        std::process::exit(1);
    }
}
