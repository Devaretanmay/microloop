use async_stream::stream;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{
        IntoResponse,
        sse::{Event, Sse},
    },
};
use reqwest::Client;
use serde_json::{Value, json};
use std::convert::Infallible;
use std::sync::{Arc, Mutex};

pub fn intercept_tool_calls(
    request_body: &Value,
    response: &mut Value,
    state_mutex: &Arc<Mutex<microloop::state::MicroloopState>>,
) {
    let empty_vec = Vec::new();
    let messages = request_body
        .get("messages")
        .and_then(|m| m.as_array())
        .unwrap_or(&empty_vec);
    let prior_outcomes = microloop::history::pair_tool_calls(messages);

    let mut parsed_tool_calls = Vec::new();
    let mut is_anthropic = false;

    if let Some(choices) = response.get_mut("choices").and_then(|c| c.as_array_mut()) {
        if let Some(first_choice) = choices.first_mut()
            && let Some(message) = first_choice.get_mut("message")
            && let Some(tool_calls) = message.get("tool_calls").and_then(|tc| tc.as_array())
        {
            for tc in tool_calls {
                if let Some(function) = tc.get("function") {
                    let name = function
                        .get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("")
                        .to_string();
                    let arguments_str = function
                        .get("arguments")
                        .and_then(|a| a.as_str())
                        .unwrap_or("{}")
                        .to_string();
                    let arguments_val: Value =
                        serde_json::from_str(&arguments_str).unwrap_or(json!({}));
                    parsed_tool_calls.push((name, arguments_str, arguments_val));
                }
            }
        }
    } else if response.get("type").and_then(|t| t.as_str()) == Some("message") {
        is_anthropic = true;
        if let Some(content) = response.get("content").and_then(|c| c.as_array()) {
            for block in content {
                if block.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
                    let name = block
                        .get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("")
                        .to_string();
                    let arguments_val = block.get("input").cloned().unwrap_or(json!({}));
                    let arguments_str =
                        serde_json::to_string(&arguments_val).unwrap_or_else(|_| "{}".to_string());
                    parsed_tool_calls.push((name, arguments_str, arguments_val));
                }
            }
        }
    }

    if parsed_tool_calls.is_empty() {
        return;
    }

    for (name, arguments_str, arguments_val) in parsed_tool_calls {
        let mut state = state_mutex.lock().unwrap();

        let empty_volatile = Vec::new();
        let mut volatile_fields = &empty_volatile;
        let mut error_cfg = None;
        let mut max_repeats = state
            .defaults
            .as_ref()
            .map(|d| d.trajectory_gate.max_repeats)
            .unwrap_or(state.max_repeats);
        let mut count_mode = state
            .defaults
            .as_ref()
            .map(|d| d.trajectory_gate.count_mode.clone())
            .unwrap_or(microloop::config::CountMode::All);

        for t_cfg in &state.tools {
            if t_cfg.name == name {
                if let Some(mr) = t_cfg.trajectory_gate.max_repeats {
                    max_repeats = mr;
                }
                if let Some(cm) = &t_cfg.trajectory_gate.count_mode {
                    count_mode = cm.clone();
                }
                volatile_fields = &t_cfg.trajectory_gate.volatile_fields;
                error_cfg = t_cfg.trajectory_gate.error_detection.as_ref();
                break;
            }
        }

        let mut current_args = arguments_val.clone();
        microloop::canonical::strip_volatile_fields(&mut current_args, volatile_fields);

        let mut match_count = 0;
        let mut error_count = 0;

        for (prior_call, prior_res) in &prior_outcomes {
            if prior_call.name == name {
                let mut prior_args = prior_call.arguments.clone();
                microloop::canonical::strip_volatile_fields(&mut prior_args, volatile_fields);
                if prior_args == current_args {
                    match_count += 1;
                    if microloop::history::is_error_response(&prior_res.content, error_cfg) {
                        error_count += 1;
                    }
                }
            }
        }

        let count = match count_mode {
            microloop::config::CountMode::All => match_count,
            microloop::config::CountMode::ErrorsOnly => error_count,
            microloop::config::CountMode::ErrorsOrIdentical => error_count.max(match_count / 2),
        };

        let res = if count >= max_repeats {
            2
        } else {
            match state.engine.validate(&arguments_str) {
                Ok(_) => 0,
                Err(msg) => {
                    state.set_error(msg);
                    state.block_result()
                }
            }
        };

        let label = match res {
            0 => "ALLOW",
            1 => "WARN",
            2 => "BLOCK_RETRY",
            3 => "BLOCK_HALT",
            _ => "UNKNOWN",
        };
        eprintln!(
            "{}",
            json!( {
                "event": "microloop_verify",
                "tool": name,
                "verdict": res,
                "label": label,
                "stateless_match_count": match_count,
                "stateless_error_count": error_count,
            })
        );

        if res != 0 {
            let err_str = if count >= max_repeats {
                format!(
                    "Trajectory blocked by stateless history. Seen {} times, {} errors. Limit: {}",
                    match_count, error_count, max_repeats
                )
            } else {
                std::str::from_utf8(&state.error_buffer)
                    .unwrap_or("Unknown error")
                    .trim_end_matches('\0')
                    .to_string()
            };

            if is_anthropic {
                if let Value::Object(resp_map) = response {
                    resp_map.insert("stop_reason".to_string(), json!("end_turn"));
                    resp_map.insert(
                        "content".to_string(),
                        json!([{
                            "type": "text",
                            "text": format!("SYSTEM INTERCEPT: Microloop blocked this action because: {}", err_str)
                        }])
                    );
                }
            } else {
                if let Some(choices) = response.get_mut("choices").and_then(|c| c.as_array_mut())
                    && let Some(first_choice) = choices.first_mut()
                    && let Some(Value::Object(msg_map)) = first_choice.get_mut("message")
                {
                    msg_map.remove("tool_calls");
                    msg_map.insert(
                        "content".to_string(),
                        json!(format!(
                            "SYSTEM INTERCEPT: Microloop blocked this action because: {}",
                            err_str
                        )),
                    );
                }
            }
            return;
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub microloop_state: Arc<Mutex<microloop::state::MicroloopState>>,
    pub target_base_url: String,
    pub api_key: String,
}

pub async fn handle_proxy_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::OriginalUri(uri): axum::extract::OriginalUri,
    axum::Json(mut body): axum::Json<Value>,
) -> impl IntoResponse {
    let client = Client::new();
    let path = uri.path();

    let upstream_path = if path == "/v1/chat/completions" {
        "/chat/completions"
    } else if path == "/v1/messages" {
        "/messages"
    } else {
        path
    };

    let url =
        if state.target_base_url.ends_with("/openai/v1") && upstream_path == "/chat/completions" {
            format!("{}/chat/completions", state.target_base_url)
        } else if state.target_base_url.ends_with("/v1") {
            format!("{}{}", state.target_base_url, upstream_path)
        } else if state.target_base_url.ends_with("/openai") {
            format!("{}{}", state.target_base_url, upstream_path)
        } else {
            format!("{}/v1{}", state.target_base_url, upstream_path)
        };

    let stream_requested = body
        .get("stream")
        .and_then(|s| s.as_bool())
        .unwrap_or(false);

    if let Value::Object(ref mut map) = body {
        map.insert("stream".to_string(), json!(false));
    }

    let mut upstream_headers = reqwest::header::HeaderMap::new();
    upstream_headers.insert("Content-Type", "application/json".parse().unwrap());
    if let Some(auth) = headers.get("authorization") {
        upstream_headers.insert("Authorization", auth.clone());
    } else if let Some(anthropic_key) = headers.get("x-api-key") {
        upstream_headers.insert("x-api-key", anthropic_key.clone());
    } else if !state.api_key.is_empty() {
        if path == "/v1/messages" {
            upstream_headers.insert("x-api-key", state.api_key.parse().unwrap());
            upstream_headers.insert("anthropic-version", "2023-06-01".parse().unwrap());
        } else {
            upstream_headers.insert(
                "Authorization",
                format!("Bearer {}", state.api_key).parse().unwrap(),
            );
        }
    }

    let resp = match client
        .post(&url)
        .headers(upstream_headers)
        .json(&body)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return (StatusCode::BAD_GATEWAY, format!("Upstream error: {}", e)).into_response();
        }
    };

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return (status, body).into_response();
    }

    let mut response_json: Value = match resp.json().await {
        Ok(j) => j,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Invalid JSON: {}", e),
            )
                .into_response();
        }
    };

    intercept_tool_calls(&body, &mut response_json, &state.microloop_state);

    if stream_requested {
        let sse_stream = stream! {
            let mut chunk1 = response_json.clone();
            if let Some(choices) = chunk1.get_mut("choices").and_then(|c| c.as_array_mut())
                && let Some(c) = choices.first_mut()
                    && let Value::Object(m) = c {
                        let msg = m.remove("message").unwrap_or(json!({}));
                        m.insert("delta".to_string(), msg);
                    }
            yield Ok::<_, Infallible>(Event::default().json_data(chunk1).unwrap());
            yield Ok::<_, Infallible>(Event::default().data("[DONE]"));
        };
        Sse::new(sse_stream).into_response()
    } else {
        axum::Json(response_json).into_response()
    }
}
