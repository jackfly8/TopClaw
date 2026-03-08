use axum::{extract::State, routing::get, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::Arc;

const SUPPORTED_ACTIONS: &[&str] = &[
    "open",
    "mouse_move",
    "mouse_click",
    "mouse_drag",
    "key_type",
    "key_press",
    "screen_capture",
    "window_list",
    "window_focus",
    "window_close",
    "app_launch",
    "app_terminate",
];

#[derive(Debug, Clone)]
struct AppState {
    platform: Arc<str>,
}

#[derive(Debug, Deserialize)]
struct ComputerUseRequest {
    action: String,
    #[serde(default)]
    params: Value,
    #[serde(default)]
    policy: ComputerUsePolicy,
    #[serde(default)]
    metadata: ComputerUseMetadata,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct ComputerUsePolicy {
    #[serde(default)]
    allowed_domains: Vec<String>,
    #[serde(default)]
    window_allowlist: Vec<String>,
    #[serde(default)]
    max_coordinate_x: Option<i64>,
    #[serde(default)]
    max_coordinate_y: Option<i64>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct ComputerUseMetadata {
    #[serde(default)]
    session_name: Option<String>,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    platform: Option<String>,
}

#[derive(Debug, Serialize)]
struct ComputerUseResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

async fn health() -> Json<Value> {
    Json(json!({
        "ok": true,
        "service": "topclaw-computer-use-stub",
        "platform": std::env::consts::OS,
        "supported_actions": SUPPORTED_ACTIONS,
    }))
}

fn build_stub_response(platform: &str, request: ComputerUseRequest) -> ComputerUseResponse {
    let data = match request.action.as_str() {
        "screen_capture" => json!({
            "stub": true,
            "platform": platform,
            "action": request.action,
            "message": "No real capture backend is installed. Implement the local sidecar for this platform.",
            "requested_path": request.params.get("path").cloned(),
            "policy": request.policy,
            "metadata": request.metadata,
        }),
        "window_list" => json!({
            "stub": true,
            "platform": platform,
            "action": request.action,
            "windows": [],
            "query": request.params.get("query").cloned(),
            "policy": request.policy,
            "metadata": request.metadata,
            "message": "No real window enumeration backend is installed."
        }),
        "window_focus" | "window_close" | "app_launch" | "app_terminate" | "mouse_move"
        | "mouse_click" | "mouse_drag" | "key_type" | "key_press" | "open" => json!({
            "stub": true,
            "platform": platform,
            "action": request.action,
            "accepted": true,
            "params": request.params,
            "policy": request.policy,
            "metadata": request.metadata,
            "message": "Protocol accepted by stub sidecar. Replace this example with a real OS backend."
        }),
        other => {
            return ComputerUseResponse {
                success: false,
                data: None,
                error: Some(format!("Unsupported action for stub sidecar: {other}")),
            };
        }
    };

    ComputerUseResponse {
        success: true,
        data: Some(data),
        error: None,
    }
}

async fn handle_action(
    State(state): State<AppState>,
    Json(request): Json<ComputerUseRequest>,
) -> Json<ComputerUseResponse> {
    Json(build_stub_response(state.platform.as_ref(), request))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let bind = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8787".to_string());
    let addr: SocketAddr = bind.parse()?;
    let state = AppState {
        platform: Arc::from(std::env::consts::OS),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/actions", post(handle_action))
        .with_state(state);

    println!("topclaw computer-use stub listening on http://{addr}");
    println!("health: http://{addr}/health");
    println!("actions: http://{addr}/v1/actions");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_window_list_returns_success_payload() {
        let response = build_stub_response(
            "linux",
            ComputerUseRequest {
                action: "window_list".to_string(),
                params: json!({ "query": "code" }),
                policy: ComputerUsePolicy {
                    window_allowlist: vec!["Visual Studio Code".to_string()],
                    ..ComputerUsePolicy::default()
                },
                metadata: ComputerUseMetadata {
                    source: Some("topclaw.browser".to_string()),
                    ..ComputerUseMetadata::default()
                },
            },
        );

        assert!(response.success);
        let data = response.data.expect("stub should return data");
        assert_eq!(
            data.get("action"),
            Some(&Value::String("window_list".to_string()))
        );
        assert_eq!(data.get("query"), Some(&Value::String("code".to_string())));
    }

    #[test]
    fn stub_rejects_unknown_actions() {
        let response = build_stub_response(
            "linux",
            ComputerUseRequest {
                action: "window_resize".to_string(),
                params: Value::Null,
                policy: ComputerUsePolicy::default(),
                metadata: ComputerUseMetadata::default(),
            },
        );

        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(
            response.error.as_deref(),
            Some("Unsupported action for stub sidecar: window_resize")
        );
    }
}
