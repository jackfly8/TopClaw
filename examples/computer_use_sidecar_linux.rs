use anyhow::Context;
use axum::{extract::State, routing::get, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use tempfile::NamedTempFile;
use tokio::process::Command;

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
    session_type: Arc<str>,
}

#[derive(Debug, Deserialize)]
struct ComputerUseRequest {
    action: String,
    #[serde(default)]
    params: Value,
    #[serde(default)]
    policy: ComputerUsePolicy,
    #[serde(rename = "metadata", default)]
    _metadata: ComputerUseMetadata,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
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

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
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

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct WindowEntry {
    window_id: String,
    desktop: String,
    host: String,
    wm_class: String,
    title: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SessionType {
    X11,
    Wayland,
    Unknown,
}

impl SessionType {
    fn detect() -> Self {
        match std::env::var("XDG_SESSION_TYPE")
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "x11" => Self::X11,
            "wayland" => Self::Wayland,
            _ => Self::Unknown,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::X11 => "x11",
            Self::Wayland => "wayland",
            Self::Unknown => "unknown",
        }
    }

    fn from_str(value: &str) -> Self {
        match value {
            "x11" => Self::X11,
            "wayland" => Self::Wayland,
            _ => Self::Unknown,
        }
    }
}

async fn health(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "ok": true,
        "service": "topclaw-computer-use-linux",
        "platform": std::env::consts::OS,
        "session_type": state.session_type.as_ref(),
        "supported_actions": SUPPORTED_ACTIONS,
        "tooling": {
            "xdotool": which::which("xdotool").is_ok(),
            "wmctrl": which::which("wmctrl").is_ok(),
            "xdg-open": which::which("xdg-open").is_ok(),
            "gnome-screenshot": which::which("gnome-screenshot").is_ok(),
            "scrot": which::which("scrot").is_ok(),
            "import": which::which("import").is_ok(),
            "pkill": which::which("pkill").is_ok(),
        }
    }))
}

fn success(data: Value) -> Json<ComputerUseResponse> {
    Json(ComputerUseResponse {
        success: true,
        data: Some(data),
        error: None,
    })
}

fn failure(message: impl Into<String>) -> Json<ComputerUseResponse> {
    Json(ComputerUseResponse {
        success: false,
        data: None,
        error: Some(message.into()),
    })
}

fn require_string<'a>(params: &'a Value, field: &str) -> anyhow::Result<&'a str> {
    params
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow::anyhow!("'{field}' is required"))
}

fn optional_string<'a>(params: &'a Value, field: &str) -> Option<&'a str> {
    params
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn optional_i64(params: &Value, field: &str) -> Option<i64> {
    params.get(field).and_then(Value::as_i64)
}

fn enforce_coordinate_limit(
    params: &Value,
    field: &str,
    max: Option<i64>,
) -> anyhow::Result<Option<i64>> {
    let value = optional_i64(params, field);
    if let (Some(actual), Some(limit)) = (value, max) {
        if actual < 0 || actual > limit {
            anyhow::bail!("'{field}' must be between 0 and {limit}");
        }
    }
    Ok(value)
}

fn host_allowed(url: &str, allowed_domains: &[String]) -> anyhow::Result<()> {
    if allowed_domains.is_empty() {
        return Ok(());
    }

    let parsed = reqwest::Url::parse(url).context("invalid URL for open action")?;
    let host = parsed
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("open action URL must include host"))?
        .to_ascii_lowercase();

    let allowed = allowed_domains.iter().any(|domain| {
        let candidate = domain.trim().trim_start_matches('.').to_ascii_lowercase();
        !candidate.is_empty() && (host == candidate || host.ends_with(&format!(".{candidate}")))
    });

    if !allowed {
        anyhow::bail!("URL host '{host}' is not in policy.allowed_domains");
    }

    Ok(())
}

fn enforce_window_allowlist(
    policy: &ComputerUsePolicy,
    title: Option<&str>,
    app: Option<&str>,
) -> anyhow::Result<()> {
    if policy.window_allowlist.is_empty() {
        return Ok(());
    }

    let title = title.unwrap_or_default().to_ascii_lowercase();
    let app = app.unwrap_or_default().to_ascii_lowercase();
    let allowed = policy.window_allowlist.iter().any(|item| {
        let needle = item.trim().to_ascii_lowercase();
        !needle.is_empty()
            && ((!title.is_empty() && title.contains(&needle))
                || (!app.is_empty() && app.contains(&needle)))
    });

    if allowed {
        Ok(())
    } else {
        anyhow::bail!("window or app is not permitted by policy.window_allowlist")
    }
}

fn parse_wmctrl_windows(stdout: &str) -> Vec<WindowEntry> {
    stdout
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let window_id = parts.next()?.to_string();
            let desktop = parts.next()?.to_string();
            let host = parts.next()?.to_string();
            let wm_class = parts.next()?.to_string();
            let title = parts.collect::<Vec<_>>().join(" ");
            Some(WindowEntry {
                window_id,
                desktop,
                host,
                wm_class,
                title,
            })
        })
        .collect()
}

fn filter_windows(windows: Vec<WindowEntry>, query: Option<&str>) -> Vec<WindowEntry> {
    let Some(query) = query.map(|value| value.to_ascii_lowercase()) else {
        return windows;
    };
    windows
        .into_iter()
        .filter(|window| {
            window.title.to_ascii_lowercase().contains(&query)
                || window.wm_class.to_ascii_lowercase().contains(&query)
        })
        .collect()
}

fn screenshot_commands(output_path: &Path) -> Vec<Vec<String>> {
    let output = output_path.to_string_lossy().into_owned();
    vec![
        vec!["gnome-screenshot".into(), "-f".into(), output.clone()],
        vec!["scrot".into(), output.clone()],
        vec!["import".into(), "-window".into(), "root".into(), output],
    ]
}

fn x11_only(action: &str, session_type: SessionType) -> anyhow::Result<()> {
    if matches!(session_type, SessionType::Wayland | SessionType::Unknown) {
        anyhow::bail!(
            "'{action}' requires an X11 session plus xdotool/wmctrl. Current session type: {}",
            session_type.as_str()
        );
    }
    Ok(())
}

async fn run_command(program: &str, args: &[String]) -> anyhow::Result<(String, String)> {
    let output = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .await
        .with_context(|| format!("failed to start '{program}'"))?;

    if output.status.success() {
        return Ok((
            String::from_utf8_lossy(&output.stdout).trim().to_string(),
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let detail = if stderr.is_empty() {
        format!("'{program}' exited with status {}", output.status)
    } else {
        stderr
    };
    anyhow::bail!("{detail}");
}

async fn run_spawn(program: &str, args: &[String]) -> anyhow::Result<u32> {
    let child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("failed to launch '{program}'"))?;
    Ok(child.id().unwrap_or_default())
}

async fn handle_open(params: &Value, policy: &ComputerUsePolicy) -> anyhow::Result<Value> {
    let url = require_string(params, "url")?;
    host_allowed(url, &policy.allowed_domains)?;
    run_command("xdg-open", &[url.to_string()]).await?;
    Ok(json!({
        "action": "open",
        "url": url,
        "opened": true,
    }))
}

async fn handle_screen_capture(params: &Value) -> anyhow::Result<Value> {
    let output_path = if let Some(path) = optional_string(params, "path") {
        let path = PathBuf::from(path);
        if !path.is_absolute() {
            anyhow::bail!("screen_capture requires an absolute 'path'");
        }
        path
    } else {
        NamedTempFile::new()
            .context("failed to create temporary screenshot file")?
            .into_temp_path()
            .keep()
            .context("failed to persist temporary screenshot file")?
    };

    let mut last_error = None;
    for command in screenshot_commands(&output_path) {
        let program = &command[0];
        let args = &command[1..];
        match run_command(program, args).await {
            Ok(_) => {
                return Ok(json!({
                    "action": "screen_capture",
                    "path": output_path,
                    "captured": true,
                }));
            }
            Err(err) => last_error = Some(err.to_string()),
        }
    }

    anyhow::bail!(
        "{}",
        last_error.unwrap_or_else(|| {
            "no Linux screenshot tool found; install gnome-screenshot, scrot, or ImageMagick"
                .to_string()
        })
    )
}

async fn handle_window_list(params: &Value) -> anyhow::Result<Value> {
    let (stdout, _) = run_command("wmctrl", &["-lx".to_string()]).await?;
    let query = optional_string(params, "query");
    let windows = filter_windows(parse_wmctrl_windows(&stdout), query);
    Ok(json!({
        "action": "window_list",
        "windows": windows,
        "count": windows.len(),
    }))
}

async fn handle_window_action(
    action: &str,
    params: &Value,
    policy: &ComputerUsePolicy,
) -> anyhow::Result<Value> {
    let window_id = optional_string(params, "window_id");
    let window_title = optional_string(params, "window_title");
    let app = optional_string(params, "app");
    enforce_window_allowlist(policy, window_title, app)?;

    let args = if let Some(window_id) = window_id {
        match action {
            "window_focus" => vec!["-ia".to_string(), window_id.to_string()],
            "window_close" => vec!["-ic".to_string(), window_id.to_string()],
            _ => anyhow::bail!("unsupported window action '{action}'"),
        }
    } else {
        let matcher = window_title.or(app).ok_or_else(|| {
            anyhow::anyhow!("'{action}' requires one of: window_id, window_title, or app")
        })?;
        match action {
            "window_focus" => vec!["-a".to_string(), matcher.to_string()],
            "window_close" => vec!["-c".to_string(), matcher.to_string()],
            _ => anyhow::bail!("unsupported window action '{action}'"),
        }
    };

    run_command("wmctrl", &args).await?;
    Ok(json!({
        "action": action,
        "window_id": window_id,
        "window_title": window_title,
        "app": app,
        "accepted": true,
    }))
}

async fn handle_mouse_action(
    action: &str,
    params: &Value,
    policy: &ComputerUsePolicy,
) -> anyhow::Result<Value> {
    match action {
        "mouse_move" => {
            let x = enforce_coordinate_limit(params, "x", policy.max_coordinate_x)?
                .ok_or_else(|| anyhow::anyhow!("'x' is required"))?;
            let y = enforce_coordinate_limit(params, "y", policy.max_coordinate_y)?
                .ok_or_else(|| anyhow::anyhow!("'y' is required"))?;
            run_command(
                "xdotool",
                &["mousemove".to_string(), x.to_string(), y.to_string()],
            )
            .await?;
            Ok(json!({ "action": action, "x": x, "y": y, "accepted": true }))
        }
        "mouse_click" => {
            let x = enforce_coordinate_limit(params, "x", policy.max_coordinate_x)?
                .ok_or_else(|| anyhow::anyhow!("'x' is required"))?;
            let y = enforce_coordinate_limit(params, "y", policy.max_coordinate_y)?
                .ok_or_else(|| anyhow::anyhow!("'y' is required"))?;
            let button = optional_string(params, "button").unwrap_or("left");
            let button_code = match button {
                "left" => "1",
                "middle" => "2",
                "right" => "3",
                other => anyhow::bail!("unsupported mouse button '{other}'"),
            };
            run_command(
                "xdotool",
                &[
                    "mousemove".to_string(),
                    x.to_string(),
                    y.to_string(),
                    "click".to_string(),
                    button_code.to_string(),
                ],
            )
            .await?;
            Ok(json!({ "action": action, "x": x, "y": y, "button": button, "accepted": true }))
        }
        "mouse_drag" => {
            let from_x = enforce_coordinate_limit(params, "from_x", policy.max_coordinate_x)?
                .ok_or_else(|| anyhow::anyhow!("'from_x' is required"))?;
            let from_y = enforce_coordinate_limit(params, "from_y", policy.max_coordinate_y)?
                .ok_or_else(|| anyhow::anyhow!("'from_y' is required"))?;
            let to_x = enforce_coordinate_limit(params, "to_x", policy.max_coordinate_x)?
                .ok_or_else(|| anyhow::anyhow!("'to_x' is required"))?;
            let to_y = enforce_coordinate_limit(params, "to_y", policy.max_coordinate_y)?
                .ok_or_else(|| anyhow::anyhow!("'to_y' is required"))?;
            run_command(
                "xdotool",
                &[
                    "mousemove".to_string(),
                    from_x.to_string(),
                    from_y.to_string(),
                    "mousedown".to_string(),
                    "1".to_string(),
                    "mousemove".to_string(),
                    to_x.to_string(),
                    to_y.to_string(),
                    "mouseup".to_string(),
                    "1".to_string(),
                ],
            )
            .await?;
            Ok(json!({
                "action": action,
                "from_x": from_x,
                "from_y": from_y,
                "to_x": to_x,
                "to_y": to_y,
                "accepted": true
            }))
        }
        _ => anyhow::bail!("unsupported mouse action '{action}'"),
    }
}

async fn handle_keyboard_action(action: &str, params: &Value) -> anyhow::Result<Value> {
    match action {
        "key_type" => {
            let text = require_string(params, "text")?;
            run_command(
                "xdotool",
                &[
                    "type".to_string(),
                    "--delay".to_string(),
                    "1".to_string(),
                    text.to_string(),
                ],
            )
            .await?;
            Ok(json!({ "action": action, "text": text, "accepted": true }))
        }
        "key_press" => {
            let key = require_string(params, "key")?;
            run_command("xdotool", &["key".to_string(), key.to_string()]).await?;
            Ok(json!({ "action": action, "key": key, "accepted": true }))
        }
        _ => anyhow::bail!("unsupported keyboard action '{action}'"),
    }
}

async fn handle_app_launch(params: &Value, policy: &ComputerUsePolicy) -> anyhow::Result<Value> {
    let app = require_string(params, "app")?;
    enforce_window_allowlist(policy, None, Some(app))?;
    let args = params
        .get("args")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let pid = run_spawn(app, &args).await?;
    Ok(json!({
        "action": "app_launch",
        "app": app,
        "args": args,
        "pid": pid,
        "accepted": true,
    }))
}

async fn handle_app_terminate(params: &Value, policy: &ComputerUsePolicy) -> anyhow::Result<Value> {
    if let Some(pid) = optional_i64(params, "pid") {
        run_command("kill", &[pid.to_string()]).await?;
        return Ok(json!({
            "action": "app_terminate",
            "pid": pid,
            "accepted": true,
        }));
    }

    let app = require_string(params, "app")?;
    enforce_window_allowlist(policy, None, Some(app))?;
    run_command("pkill", &[app.to_string()]).await?;
    Ok(json!({
        "action": "app_terminate",
        "app": app,
        "accepted": true,
    }))
}

async fn dispatch(session_type: SessionType, request: ComputerUseRequest) -> anyhow::Result<Value> {
    match request.action.as_str() {
        "open" => handle_open(&request.params, &request.policy).await,
        "screen_capture" => handle_screen_capture(&request.params).await,
        "window_list" => {
            x11_only("window_list", session_type)?;
            handle_window_list(&request.params).await
        }
        "window_focus" | "window_close" => {
            x11_only(&request.action, session_type)?;
            handle_window_action(&request.action, &request.params, &request.policy).await
        }
        "mouse_move" | "mouse_click" | "mouse_drag" => {
            x11_only(&request.action, session_type)?;
            handle_mouse_action(&request.action, &request.params, &request.policy).await
        }
        "key_type" | "key_press" => {
            x11_only(&request.action, session_type)?;
            handle_keyboard_action(&request.action, &request.params).await
        }
        "app_launch" => handle_app_launch(&request.params, &request.policy).await,
        "app_terminate" => handle_app_terminate(&request.params, &request.policy).await,
        other => anyhow::bail!("unsupported action '{other}'"),
    }
}

async fn handle_action(
    State(state): State<AppState>,
    Json(request): Json<ComputerUseRequest>,
) -> Json<ComputerUseResponse> {
    match dispatch(SessionType::from_str(state.session_type.as_ref()), request).await {
        Ok(data) => success(data),
        Err(err) => failure(err.to_string()),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let bind = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8788".to_string());
    let addr: SocketAddr = bind.parse()?;
    let state = AppState {
        session_type: Arc::from(SessionType::detect().as_str()),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/actions", post(handle_action))
        .with_state(state);

    println!("topclaw computer-use linux sidecar listening on http://{addr}");
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
    fn parse_wmctrl_output_extracts_windows() {
        let windows = parse_wmctrl_windows(
            "0x04800007  0 host code.Code Visual Studio Code\n0x05200003  0 host google-chrome.Google-chrome Inbox - Gmail",
        );

        assert_eq!(windows.len(), 2);
        assert_eq!(
            windows[0],
            WindowEntry {
                window_id: "0x04800007".to_string(),
                desktop: "0".to_string(),
                host: "host".to_string(),
                wm_class: "code.Code".to_string(),
                title: "Visual Studio Code".to_string(),
            }
        );
    }

    #[test]
    fn filter_windows_matches_title_and_class() {
        let windows = vec![
            WindowEntry {
                window_id: "0x1".to_string(),
                desktop: "0".to_string(),
                host: "host".to_string(),
                wm_class: "code.Code".to_string(),
                title: "Visual Studio Code".to_string(),
            },
            WindowEntry {
                window_id: "0x2".to_string(),
                desktop: "0".to_string(),
                host: "host".to_string(),
                wm_class: "firefox.Firefox".to_string(),
                title: "Mozilla Firefox".to_string(),
            },
        ];

        assert_eq!(filter_windows(windows.clone(), Some("code")).len(), 1);
        assert_eq!(filter_windows(windows, Some("firefox")).len(), 1);
    }

    #[test]
    fn allowlist_enforcement_rejects_unlisted_targets() {
        let policy = ComputerUsePolicy {
            window_allowlist: vec!["visual studio code".to_string()],
            ..ComputerUsePolicy::default()
        };

        assert!(enforce_window_allowlist(&policy, Some("Visual Studio Code"), None).is_ok());
        assert!(enforce_window_allowlist(&policy, Some("Mozilla Firefox"), None).is_err());
    }

    #[test]
    fn open_action_respects_allowed_domains() {
        let allowed = vec!["example.com".to_string()];
        assert!(host_allowed("https://example.com/path", &allowed).is_ok());
        assert!(host_allowed("https://sub.example.com/path", &allowed).is_ok());
        assert!(host_allowed("https://evil.com/path", &allowed).is_err());
    }

    #[test]
    fn coordinate_limits_are_enforced() {
        let params = json!({ "x": 200, "y": 40 });
        assert_eq!(
            enforce_coordinate_limit(&params, "x", Some(500)).unwrap(),
            Some(200)
        );
        assert!(enforce_coordinate_limit(&params, "x", Some(100)).is_err());
    }

    #[test]
    fn wayland_sessions_are_rejected_for_x11_only_actions() {
        let result = x11_only("mouse_move", SessionType::Wayland);
        assert!(result.is_err());
    }
}
