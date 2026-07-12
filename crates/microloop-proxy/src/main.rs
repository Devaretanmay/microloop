use axum::{
    Router,
    routing::{get, post},
};
use std::env;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

mod proxy;
mod ccr;

#[tokio::main]
async fn main() {
    println!("Starting Microloop Proxy...");

    let yaml = match std::fs::read_to_string("microloop.yaml") {
        Ok(y) => y,
        Err(_) => {
            let default_yaml = microloop::config::MicroloopConfig::default_yaml();
            eprintln!(
                "microloop.yaml not found — writing default config and continuing."
            );
            let _ = std::fs::write("microloop.yaml", &default_yaml);
            default_yaml
        }
    };

    let microloop_state = microloop::state::MicroloopState::new(&yaml).unwrap_or_else(|e| {
        eprintln!("Failed to initialize Microloop core: {}", e);
        std::process::exit(1);
    });
    println!("Microloop core initialized.");

    let microloop_state_ref = Arc::new(Mutex::new(microloop_state));

    let ccr_db_path = env::var("CCR_DB_PATH").unwrap_or_else(|_| "microloop_ccr.db".to_string());
    let ccr_store = match crate::ccr::CcrStore::new(&ccr_db_path) {
        Ok(store) => Arc::new(store),
        Err(e) => {
            eprintln!("Failed to initialize CCR store at {}: {}", ccr_db_path, e);
            std::process::exit(1);
        }
    };

    let learn_yaml_path = env::var("MICROLOOP_LEARN_PATH")
        .unwrap_or_else(|_| "microloop_learned.yaml".to_string());

    // Set up the trajectory collector with a hot-reload callback
    let microloop_state_for_cb = microloop_state_ref.clone();
    let trajectory_collector = microloop_learn::collector::TrajectoryCollector::new()
        .with_max_buffer(200)
        .with_min_analysis(15)
        .with_policy_path(std::path::PathBuf::from(&learn_yaml_path))
        .with_policy_callback(move |yaml: &str| {
            // Hot-reload: when learn generates a new policy, reload it into
            // the microloop state so thresholds update without restarting.
            //
            // Use try_lock instead of lock to prevent deadlock with concurrent
            // proxy requests. The lock sequence in intercept_tool_calls is:
            //   lock(microloop_state) → drop → lock(trajectory_collector) → callback → lock(microloop_state)
            // If two concurrent requests interleave, we'd get a classic A-B / B-A
            // deadlock. try_lock skips the hot-reload if the state is busy; the
            // next flush will retry.
            let yaml_to_send = if yaml.contains("---") {
                yaml.trim_start_matches("---\n").to_string()
            } else {
                yaml.to_string()
            };

            // Merge learned config with existing config
            match microloop::state::MicroloopState::new(&yaml_to_send) {
                Ok(new_state) => {
                    match microloop_state_for_cb.try_lock() {
                        Ok(mut state) => {
                            // Update thresholds from the learned policy
                            state.sensitivity = new_state.sensitivity;
                            state.max_repeats = new_state.max_repeats;
                            state.ignore_args = new_state.ignore_args;
                            state.history_window = new_state.history_window;
                            state.defaults = new_state.defaults;
                            state.tools = new_state.tools;
                            eprintln!(
                                "[Learn] Hot-reloaded {} tool configs from learned policy. Thresholds updated.",
                                state.tools.len(),
                            );
                        }
                        Err(_) => {
                            eprintln!(
                                "[Learn] Hot-reload skipped: microloop_state busy. Will retry on next flush."
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[Learn] Failed to parse generated policy: {e}");
                }
            }
        });

    let state = proxy::AppState {
        microloop_state: microloop_state_ref,
        target_base_url: env::var("TARGET_API_URL")
            .unwrap_or_else(|_| "https://api.openai.com".to_string()),
        api_key: env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
            eprintln!("Warning: OPENAI_API_KEY not set, upstream requests will likely 401");
            String::new()
        }),
        ccr_store,
        trajectory_summary: Arc::new(Mutex::new(proxy::RollingSummary::new(5))),
        last_verdict: Arc::new(Mutex::new(None)),
        trajectory_collector: Arc::new(Mutex::new(trajectory_collector)),
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Microloop proxy listening on {}", addr);

    let app = Router::new()
        .route("/v1/chat/completions", post(proxy::handle_proxy_request))
        .route("/v1/messages", post(proxy::handle_proxy_request))
        .route("/", get(|| async { "Microloop Proxy Running" }))
        .route("/status", get(proxy::status_handler))
        .with_state(state);

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
