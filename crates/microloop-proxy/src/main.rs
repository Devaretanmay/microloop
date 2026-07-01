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

    let state = proxy::AppState {
        microloop_state: std::sync::Arc::new(std::sync::Mutex::new(microloop_state)),
        target_base_url: env::var("TARGET_API_URL")
            .unwrap_or_else(|_| "https://api.openai.com".to_string()),
        api_key: env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
            eprintln!("Warning: OPENAI_API_KEY not set, upstream requests will likely 401");
            String::new()
        }),
    };

    let app = Router::new()
        .route("/v1/chat/completions", post(proxy::handle_proxy_request))
        .route("/v1/messages", post(proxy::handle_proxy_request))
        .route("/", get(|| async { "Microloop Proxy Running" }))
        .with_state(state.clone());

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
