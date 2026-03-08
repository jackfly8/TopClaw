#![cfg_attr(not(target_os = "macos"), allow(dead_code, unused_imports))]

use anyhow::Context;
use axum::{extract::State, routing::get, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::Stdio;

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

const IMPLEMENTED_ACTIONS: &[&str] = &[
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

#[derive(Debug, Clone, Default)]
struct AppState;

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

async fn health(_: State<AppState>) -> Json<Value> {
    Json(json!({
        "ok": true,
        "service": "topclaw-computer-use-macos",
        "platform": std::env::consts::OS,
        "supported_actions": SUPPORTED_ACTIONS,
        "implemented_actions": IMPLEMENTED_ACTIONS,
        "tooling": {
            "open": which::which("open").is_ok(),
            "osascript": which::which("osascript").is_ok(),
            "screencapture": which::which("screencapture").is_ok(),
            "cliclick": which::which("cliclick").is_ok(),
            "pkill": which::which("pkill").is_ok(),
        },
        "limitations": [
            "mouse actions require cliclick to be installed",
            "window enumeration and focus/close require Accessibility permissions for osascript/System Events"
        ]
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

fn escape_single_quotes(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
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

fn cliclick_button(button: &str) -> anyhow::Result<&'static str> {
    match button {
        "left" => Ok("c"),
        "right" => Ok("rc"),
        "middle" => Ok("mc"),
        other => anyhow::bail!("unsupported mouse button '{other}'"),
    }
}

fn mac_key_code(key: &str) -> Option<u16> {
    match key.trim().to_ascii_lowercase().as_str() {
        "enter" | "return" => Some(36),
        "tab" => Some(48),
        "space" => Some(49),
        "escape" | "esc" => Some(53),
        "delete" | "backspace" => Some(51),
        "up" | "arrowup" => Some(126),
        "down" | "arrowdown" => Some(125),
        "left" | "arrowleft" => Some(123),
        "right" | "arrowright" => Some(124),
        "home" => Some(115),
        "end" => Some(119),
        "pageup" => Some(116),
        "pagedown" => Some(121),
        "f1" => Some(122),
        "f2" => Some(120),
        "f3" => Some(99),
        "f4" => Some(118),
        "f5" => Some(96),
        "f6" => Some(97),
        "f7" => Some(98),
        "f8" => Some(100),
        "f9" => Some(101),
        "f10" => Some(109),
        "f11" => Some(103),
        "f12" => Some(111),
        _ => None,
    }
}

#[cfg(target_os = "macos")]
async fn run_command(program: &str, args: &[String]) -> anyhow::Result<String> {
    let output = tokio::process::Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .await
        .with_context(|| format!("failed to start '{program}'"))?;

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let detail = if stderr.is_empty() {
        format!("'{program}' exited with status {}", output.status)
    } else {
        stderr
    };
    anyhow::bail!("{detail}");
}

#[cfg(not(target_os = "macos"))]
async fn run_command(_program: &str, _args: &[String]) -> anyhow::Result<String> {
    anyhow::bail!("computer_use_sidecar_macos can only run on macOS");
}

#[cfg(target_os = "macos")]
async fn run_spawn(program: &str, args: &[String]) -> anyhow::Result<u32> {
    let child = tokio::process::Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("failed to launch '{program}'"))?;
    Ok(child.id().unwrap_or_default())
}

#[cfg(not(target_os = "macos"))]
async fn run_spawn(_program: &str, _args: &[String]) -> anyhow::Result<u32> {
    anyhow::bail!("computer_use_sidecar_macos can only run on macOS");
}

async fn run_osascript(script: &str) -> anyhow::Result<String> {
    run_command(
        "osascript",
        &[
            "-l".to_string(),
            "JavaScript".to_string(),
            "-e".to_string(),
            script.to_string(),
        ],
    )
    .await
}

async fn run_cliclick(args: &[String]) -> anyhow::Result<String> {
    if which::which("cliclick").is_err() {
        anyhow::bail!(
            "cliclick is required for macOS mouse actions; install it with 'brew install cliclick'"
        );
    }
    run_command("cliclick", args).await
}

async fn handle_open(params: &Value, policy: &ComputerUsePolicy) -> anyhow::Result<Value> {
    let url = require_string(params, "url")?;
    host_allowed(url, &policy.allowed_domains)?;
    run_command("open", &[url.to_string()]).await?;
    Ok(json!({
        "action": "open",
        "url": url,
        "opened": true,
    }))
}

async fn handle_screen_capture(params: &Value) -> anyhow::Result<Value> {
    let output_path = optional_string(params, "path")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp/topclaw_capture.png"));
    let output = output_path.to_string_lossy().into_owned();
    run_command("screencapture", &["-x".to_string(), output.clone()]).await?;
    Ok(json!({
        "action": "screen_capture",
        "path": output,
        "captured": true,
    }))
}

async fn handle_window_list(params: &Value, policy: &ComputerUsePolicy) -> anyhow::Result<Value> {
    let query = optional_string(params, "query")
        .unwrap_or_default()
        .to_ascii_lowercase();
    let script = r#"
ObjC.import('stdlib');
const se = Application('System Events');
const windows = [];
for (const proc of se.applicationProcesses()) {
  try {
    if (!proc.backgroundOnly()) {
      for (const win of proc.windows()) {
        windows.push({
          app: proc.name(),
          pid: proc.unixId(),
          window_title: win.name(),
        });
      }
    }
  } catch (error) {}
}
JSON.stringify(windows);
"#;
    let stdout = run_osascript(script).await?;
    let mut windows = serde_json::from_str::<Vec<Value>>(&stdout).unwrap_or_default();

    if !query.is_empty() {
        windows.retain(|window| {
            let title = window
                .get("window_title")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_ascii_lowercase();
            let app = window
                .get("app")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_ascii_lowercase();
            title.contains(&query) || app.contains(&query)
        });
    }

    if !policy.window_allowlist.is_empty() {
        windows.retain(|window| {
            let title = window.get("window_title").and_then(Value::as_str);
            let app = window.get("app").and_then(Value::as_str);
            enforce_window_allowlist(policy, title, app).is_ok()
        });
    }

    Ok(json!({
        "action": "window_list",
        "windows": windows,
        "count": windows.len(),
    }))
}

async fn handle_window_focus(params: &Value, policy: &ComputerUsePolicy) -> anyhow::Result<Value> {
    let app = optional_string(params, "app")
        .or_else(|| optional_string(params, "window_title"))
        .ok_or_else(|| anyhow::anyhow!("'window_focus' requires 'app' or 'window_title'"))?;
    enforce_window_allowlist(
        policy,
        optional_string(params, "window_title"),
        optional_string(params, "app"),
    )?;
    let app = escape_single_quotes(app);
    let script = format!(
        "const app = Application('{}'); app.activate(); JSON.stringify({{accepted:true, app:'{}'}});",
        app, app
    );
    let stdout = run_osascript(&script).await?;
    let value = serde_json::from_str::<Value>(&stdout)
        .unwrap_or_else(|_| json!({"accepted": true, "app": app}));
    Ok(json!({
        "action": "window_focus",
        "accepted": true,
        "window": value,
    }))
}

async fn handle_window_close(params: &Value, policy: &ComputerUsePolicy) -> anyhow::Result<Value> {
    let app = optional_string(params, "app")
        .ok_or_else(|| anyhow::anyhow!("'window_close' currently requires 'app' on macOS"))?;
    let window_title = optional_string(params, "window_title");
    enforce_window_allowlist(policy, window_title, Some(app))?;
    let app_escaped = escape_single_quotes(app);
    let title_filter = window_title.map(escape_single_quotes);
    let script = if let Some(title) = title_filter {
        format!(
            r#"
const se = Application('System Events');
const proc = se.processes.byName('{}');
const wins = proc.windows.whose({{name: {{_contains: '{}'}}}})();
if (!wins.length) throw new Error('window not found');
wins[0].actions.byName('AXClose').perform();
JSON.stringify({{accepted:true, app:'{}', window_title:'{}'}});
"#,
            app_escaped, title, app_escaped, title
        )
    } else {
        format!(
            r#"
const app = Application('{}');
app.quit();
JSON.stringify({{accepted:true, app:'{}'}});
"#,
            app_escaped, app_escaped
        )
    };
    let stdout = run_osascript(&script).await?;
    let value = serde_json::from_str::<Value>(&stdout)
        .unwrap_or_else(|_| json!({"accepted": true, "app": app}));
    Ok(json!({
        "action": "window_close",
        "accepted": true,
        "window": value,
    }))
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

    let pid = if args.is_empty() {
        run_command("open", &["-a".to_string(), app.to_string()]).await?;
        0
    } else {
        let mut open_args = vec!["-a".to_string(), app.to_string(), "--args".to_string()];
        open_args.extend(args.clone());
        run_command("open", &open_args).await?;
        0
    };

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
    let app_escaped = escape_single_quotes(app);
    let script = format!(
        "const app = Application('{}'); app.quit(); JSON.stringify({{accepted:true, app:'{}'}});",
        app_escaped, app_escaped
    );
    if let Err(primary_err) = run_osascript(&script).await {
        run_command("pkill", &[app.to_string()])
            .await
            .with_context(|| {
                format!("failed to quit app via osascript ({primary_err}) and pkill")
            })?;
    }
    Ok(json!({
        "action": "app_terminate",
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
            run_cliclick(&[format!("m:{x},{y}")]).await?;
            Ok(json!({ "action": action, "x": x, "y": y, "accepted": true }))
        }
        "mouse_click" => {
            let x = enforce_coordinate_limit(params, "x", policy.max_coordinate_x)?
                .ok_or_else(|| anyhow::anyhow!("'x' is required"))?;
            let y = enforce_coordinate_limit(params, "y", policy.max_coordinate_y)?
                .ok_or_else(|| anyhow::anyhow!("'y' is required"))?;
            let button = optional_string(params, "button").unwrap_or("left");
            run_cliclick(&[format!("{}:{x},{y}", cliclick_button(button)?)]).await?;
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
            run_cliclick(&[format!("dd:{from_x},{from_y}"), format!("du:{to_x},{to_y}")]).await?;
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
            let escaped = escape_single_quotes(text);
            let script = format!(
                "Application('System Events').keystroke('{}'); JSON.stringify({{accepted:true}});",
                escaped
            );
            run_osascript(&script).await?;
            Ok(json!({ "action": action, "text": text, "accepted": true }))
        }
        "key_press" => {
            let key = require_string(params, "key")?;
            let script = if let Some(code) = mac_key_code(key) {
                format!(
                    "Application('System Events').keyCode({}); JSON.stringify({{accepted:true, key:'{}'}});",
                    code,
                    escape_single_quotes(key)
                )
            } else if key.chars().count() == 1 {
                format!(
                    "Application('System Events').keystroke('{}'); JSON.stringify({{accepted:true, key:'{}'}});",
                    escape_single_quotes(key),
                    escape_single_quotes(key)
                )
            } else {
                anyhow::bail!("unsupported key '{key}'");
            };
            run_osascript(&script).await?;
            Ok(json!({ "action": action, "key": key, "accepted": true }))
        }
        _ => anyhow::bail!("unsupported keyboard action '{action}'"),
    }
}

async fn dispatch(request: ComputerUseRequest) -> anyhow::Result<Value> {
    match request.action.as_str() {
        "open" => handle_open(&request.params, &request.policy).await,
        "mouse_move" | "mouse_click" | "mouse_drag" => {
            handle_mouse_action(&request.action, &request.params, &request.policy).await
        }
        "key_type" | "key_press" => handle_keyboard_action(&request.action, &request.params).await,
        "screen_capture" => handle_screen_capture(&request.params).await,
        "window_list" => handle_window_list(&request.params, &request.policy).await,
        "window_focus" => handle_window_focus(&request.params, &request.policy).await,
        "window_close" => handle_window_close(&request.params, &request.policy).await,
        "app_launch" => handle_app_launch(&request.params, &request.policy).await,
        "app_terminate" => handle_app_terminate(&request.params, &request.policy).await,
        other => anyhow::bail!("unsupported action '{other}'"),
    }
}

async fn handle_action(
    _: State<AppState>,
    Json(request): Json<ComputerUseRequest>,
) -> Json<ComputerUseResponse> {
    match dispatch(request).await {
        Ok(data) => success(data),
        Err(err) => failure(err.to_string()),
    }
}

#[cfg(target_os = "macos")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let bind = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8790".to_string());
    let addr: SocketAddr = bind.parse()?;
    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/actions", post(handle_action))
        .with_state(AppState);

    println!("topclaw computer-use macos sidecar listening on http://{addr}");
    println!("health: http://{addr}/health");
    println!("actions: http://{addr}/v1/actions");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("computer_use_sidecar_macos can only run on macOS");
    std::process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_allowlist_respects_subdomains() {
        let allowed = vec!["example.com".to_string()];
        assert!(host_allowed("https://example.com", &allowed).is_ok());
        assert!(host_allowed("https://sub.example.com", &allowed).is_ok());
        assert!(host_allowed("https://evil.com", &allowed).is_err());
    }

    #[test]
    fn window_allowlist_rejects_unlisted_apps() {
        let policy = ComputerUsePolicy {
            window_allowlist: vec!["code".to_string()],
            ..ComputerUsePolicy::default()
        };
        assert!(enforce_window_allowlist(&policy, Some("Visual Studio Code"), None).is_ok());
        assert!(enforce_window_allowlist(&policy, Some("Notepad"), None).is_err());
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
    fn cliclick_button_maps_known_buttons() {
        assert_eq!(cliclick_button("left").unwrap(), "c");
        assert_eq!(cliclick_button("right").unwrap(), "rc");
        assert!(cliclick_button("extra").is_err());
    }

    #[test]
    fn mac_key_code_maps_common_keys() {
        assert_eq!(mac_key_code("enter"), Some(36));
        assert_eq!(mac_key_code("ArrowLeft"), Some(123));
        assert_eq!(mac_key_code("f12"), Some(111));
        assert_eq!(mac_key_code("unknown"), None);
    }
}
