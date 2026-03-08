#![cfg_attr(not(target_os = "windows"), allow(dead_code, unused_imports))]

use anyhow::Context;
use axum::{extract::State, routing::get, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::net::SocketAddr;
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
        "service": "topclaw-computer-use-windows",
        "platform": std::env::consts::OS,
        "supported_actions": SUPPORTED_ACTIONS,
        "implemented_actions": IMPLEMENTED_ACTIONS,
        "limitations": [
            "uses PowerShell/.NET helpers for screenshot capture and process/window enumeration",
            "uses user32.dll for mouse and keyboard input injection"
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

fn escape_ps_single_quoted(value: &str) -> String {
    value.replace('\'', "''")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WindowTarget {
    window_id: Option<String>,
    window_title: Option<String>,
    app: Option<String>,
}

fn extract_window_target(params: &Value, action: &str) -> anyhow::Result<WindowTarget> {
    let target = WindowTarget {
        window_id: optional_string(params, "window_id").map(ToString::to_string),
        window_title: optional_string(params, "window_title").map(ToString::to_string),
        app: optional_string(params, "app").map(ToString::to_string),
    };

    if target.window_id.is_none() && target.window_title.is_none() && target.app.is_none() {
        anyhow::bail!("'{action}' requires one of: window_id, window_title, or app");
    }

    Ok(target)
}

fn build_window_selector_script(target: &WindowTarget) -> String {
    let mut script = String::from(
        r#"$proc = Get-Process |
  Where-Object { $_.MainWindowTitle -and $_.MainWindowTitle.Trim().Length -gt 0 }
"#,
    );

    if let Some(window_id) = target.window_id.as_deref() {
        let window_id = escape_ps_single_quoted(window_id);
        script.push_str(&format!(
            "  | Where-Object {{ $_.MainWindowHandle.ToString() -eq '{window_id}' }}\n"
        ));
    }
    if let Some(window_title) = target.window_title.as_deref() {
        let window_title = escape_ps_single_quoted(window_title);
        script.push_str(&format!(
            "  | Where-Object {{ $_.MainWindowTitle -like '*{window_title}*' }}\n"
        ));
    }
    if let Some(app) = target.app.as_deref() {
        let app = escape_ps_single_quoted(app);
        script.push_str(&format!(
            "  | Where-Object {{ $_.ProcessName -like '*{app}*' }}\n"
        ));
    }

    script.push_str(
        r#"  | Select-Object -First 1 Id, ProcessName, MainWindowTitle, MainWindowHandle
if (-not $proc) { throw 'window not found' }
"#,
    );
    script
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

fn mouse_button_flag(button: &str, down: bool) -> anyhow::Result<u32> {
    match (button, down) {
        ("left", true) => Ok(0x0002),
        ("left", false) => Ok(0x0004),
        ("right", true) => Ok(0x0008),
        ("right", false) => Ok(0x0010),
        ("middle", true) => Ok(0x0020),
        ("middle", false) => Ok(0x0040),
        (other, _) => anyhow::bail!("unsupported mouse button '{other}'"),
    }
}

fn normalize_key_name(key: &str) -> String {
    key.trim().to_ascii_lowercase()
}

fn named_virtual_key(key: &str) -> Option<u16> {
    match normalize_key_name(key).as_str() {
        "enter" | "return" => Some(0x0D),
        "tab" => Some(0x09),
        "escape" | "esc" => Some(0x1B),
        "space" => Some(0x20),
        "backspace" => Some(0x08),
        "delete" | "del" => Some(0x2E),
        "insert" | "ins" => Some(0x2D),
        "home" => Some(0x24),
        "end" => Some(0x23),
        "pageup" => Some(0x21),
        "pagedown" => Some(0x22),
        "up" | "arrowup" => Some(0x26),
        "down" | "arrowdown" => Some(0x28),
        "left" | "arrowleft" => Some(0x25),
        "right" | "arrowright" => Some(0x27),
        "f1" => Some(0x70),
        "f2" => Some(0x71),
        "f3" => Some(0x72),
        "f4" => Some(0x73),
        "f5" => Some(0x74),
        "f6" => Some(0x75),
        "f7" => Some(0x76),
        "f8" => Some(0x77),
        "f9" => Some(0x78),
        "f10" => Some(0x79),
        "f11" => Some(0x7A),
        "f12" => Some(0x7B),
        _ => None,
    }
}

#[cfg(target_os = "windows")]
mod native_input {
    use super::{mouse_button_flag, named_virtual_key};
    use anyhow::Context;
    use std::mem::size_of;

    const KEYEVENTF_KEYUP: u32 = 0x0002;
    const KEYEVENTF_UNICODE: u32 = 0x0004;
    const MOUSEEVENTF_MOVE: u32 = 0x0001;
    const INPUT_KEYBOARD: u32 = 1;

    #[link(name = "user32")]
    unsafe extern "system" {
        fn SetCursorPos(x: i32, y: i32) -> i32;
        fn mouse_event(dwFlags: u32, dx: u32, dy: u32, dwData: u32, dwExtraInfo: usize);
        fn SendInput(c_inputs: u32, p_inputs: *const Input, cb_size: i32) -> u32;
        fn VkKeyScanW(ch: u16) -> i16;
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct MouseInput {
        dx: i32,
        dy: i32,
        mouse_data: u32,
        dw_flags: u32,
        time: u32,
        dw_extra_info: usize,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct KeyboardInput {
        w_vk: u16,
        w_scan: u16,
        dw_flags: u32,
        time: u32,
        dw_extra_info: usize,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct HardwareInput {
        u_msg: u32,
        w_param_l: u16,
        w_param_h: u16,
    }

    #[repr(C)]
    union InputUnion {
        mi: MouseInput,
        ki: KeyboardInput,
        hi: HardwareInput,
    }

    #[repr(C)]
    struct Input {
        input_type: u32,
        union: InputUnion,
    }

    fn set_cursor_pos(x: i64, y: i64) -> anyhow::Result<()> {
        let x = i32::try_from(x).context("x coordinate exceeds i32 range")?;
        let y = i32::try_from(y).context("y coordinate exceeds i32 range")?;
        let ok = unsafe {
            // SAFETY: SetCursorPos is called with primitive coordinates only and no borrowed memory.
            SetCursorPos(x, y)
        };
        if ok == 0 {
            anyhow::bail!("SetCursorPos failed");
        }
        Ok(())
    }

    fn send_mouse_flag(flag: u32) {
        unsafe {
            // SAFETY: mouse_event is called with a flag-only event and no borrowed pointers.
            mouse_event(flag, 0, 0, 0, 0);
        }
    }

    fn send_inputs(inputs: &[Input]) -> anyhow::Result<()> {
        let sent = unsafe {
            // SAFETY: SendInput reads a contiguous array of INPUT structs allocated in this function.
            // The pointer is valid for `inputs.len()` elements for the duration of the call.
            SendInput(
                u32::try_from(inputs.len()).context("too many input events")?,
                inputs.as_ptr(),
                i32::try_from(size_of::<Input>()).context("INPUT size exceeds i32 range")?,
            )
        };
        if sent != u32::try_from(inputs.len()).unwrap_or(0) {
            anyhow::bail!("SendInput failed");
        }
        Ok(())
    }

    fn keyboard_input(w_vk: u16, w_scan: u16, dw_flags: u32) -> Input {
        Input {
            input_type: INPUT_KEYBOARD,
            union: InputUnion {
                ki: KeyboardInput {
                    w_vk,
                    w_scan,
                    dw_flags,
                    time: 0,
                    dw_extra_info: 0,
                },
            },
        }
    }

    fn send_key_event(vk: u16, flags: u32) -> anyhow::Result<()> {
        send_inputs(&[keyboard_input(vk, 0, flags)])
    }

    fn press_shifted_virtual_key(vk: u16, shifted: bool) -> anyhow::Result<()> {
        if shifted {
            send_key_event(0x10, 0)?;
        }
        send_key_event(vk, 0)?;
        send_key_event(vk, KEYEVENTF_KEYUP)?;
        if shifted {
            send_key_event(0x10, KEYEVENTF_KEYUP)?;
        }
        Ok(())
    }

    pub fn mouse_move(x: i64, y: i64) -> anyhow::Result<()> {
        set_cursor_pos(x, y)
    }

    pub fn mouse_click(x: i64, y: i64, button: &str) -> anyhow::Result<()> {
        set_cursor_pos(x, y)?;
        send_mouse_flag(mouse_button_flag(button, true)?);
        send_mouse_flag(mouse_button_flag(button, false)?);
        Ok(())
    }

    pub fn mouse_drag(from_x: i64, from_y: i64, to_x: i64, to_y: i64) -> anyhow::Result<()> {
        set_cursor_pos(from_x, from_y)?;
        send_mouse_flag(mouse_button_flag("left", true)?);
        set_cursor_pos(to_x, to_y)?;
        send_mouse_flag(MOUSEEVENTF_MOVE);
        send_mouse_flag(mouse_button_flag("left", false)?);
        Ok(())
    }

    fn send_unicode_text(text: &str) -> anyhow::Result<()> {
        let mut inputs = Vec::with_capacity(text.encode_utf16().count() * 2);
        for unit in text.encode_utf16() {
            inputs.push(keyboard_input(0, unit, KEYEVENTF_UNICODE));
            inputs.push(keyboard_input(0, unit, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP));
        }
        send_inputs(&inputs)
    }

    pub fn key_type(text: &str) -> anyhow::Result<()> {
        send_unicode_text(text)
    }

    pub fn key_press(key: &str) -> anyhow::Result<()> {
        if let Some(vk) = named_virtual_key(key) {
            return press_shifted_virtual_key(vk, false);
        }

        let mut chars = key.chars();
        let Some(ch) = chars.next() else {
            anyhow::bail!("'key' is required");
        };
        if chars.next().is_some() {
            anyhow::bail!("unsupported key '{key}'; use a named key or single character");
        }

        if ch as u32 > 0xFFFF {
            anyhow::bail!("unsupported character '{ch}'");
        }

        let vk = unsafe {
            // SAFETY: VkKeyScanW reads a single UTF-16 scalar value and returns a small integer mapping.
            VkKeyScanW(ch as u16)
        };
        if vk == -1 {
            return send_unicode_text(&ch.to_string());
        }

        let vk_code = (vk & 0x00FF) as u16;
        let shift_state = ((vk >> 8) & 0x00FF) as u8;
        let shifted = (shift_state & 0x01) != 0;
        press_shifted_virtual_key(vk_code, shifted)
    }
}

#[cfg(not(target_os = "windows"))]
mod native_input {
    pub fn mouse_move(_x: i64, _y: i64) -> anyhow::Result<()> {
        anyhow::bail!("computer_use_sidecar_windows can only run on Windows");
    }

    pub fn mouse_click(_x: i64, _y: i64, _button: &str) -> anyhow::Result<()> {
        anyhow::bail!("computer_use_sidecar_windows can only run on Windows");
    }

    pub fn mouse_drag(_from_x: i64, _from_y: i64, _to_x: i64, _to_y: i64) -> anyhow::Result<()> {
        anyhow::bail!("computer_use_sidecar_windows can only run on Windows");
    }

    pub fn key_type(_text: &str) -> anyhow::Result<()> {
        anyhow::bail!("computer_use_sidecar_windows can only run on Windows");
    }

    pub fn key_press(_key: &str) -> anyhow::Result<()> {
        anyhow::bail!("computer_use_sidecar_windows can only run on Windows");
    }
}

#[cfg(target_os = "windows")]
async fn run_powershell(script: &str) -> anyhow::Result<String> {
    let output = tokio::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .stdin(Stdio::null())
        .output()
        .await
        .context("failed to start powershell")?;

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let detail = if stderr.is_empty() {
        format!("powershell exited with status {}", output.status)
    } else {
        stderr
    };
    anyhow::bail!("{detail}");
}

#[cfg(not(target_os = "windows"))]
async fn run_powershell(_script: &str) -> anyhow::Result<String> {
    anyhow::bail!("computer_use_sidecar_windows can only run on Windows");
}

#[cfg(target_os = "windows")]
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

#[cfg(not(target_os = "windows"))]
async fn run_spawn(_program: &str, _args: &[String]) -> anyhow::Result<u32> {
    anyhow::bail!("computer_use_sidecar_windows can only run on Windows");
}

async fn handle_open(params: &Value, policy: &ComputerUsePolicy) -> anyhow::Result<Value> {
    let url = require_string(params, "url")?;
    host_allowed(url, &policy.allowed_domains)?;
    let escaped = escape_ps_single_quoted(url);
    run_powershell(&format!("Start-Process '{escaped}' | Out-Null")).await?;
    Ok(json!({
        "action": "open",
        "url": url,
        "opened": true,
    }))
}

async fn handle_window_list(params: &Value, policy: &ComputerUsePolicy) -> anyhow::Result<Value> {
    let query = optional_string(params, "query").unwrap_or_default();
    let script = r#"
$items = Get-Process |
  Where-Object { $_.MainWindowTitle -and $_.MainWindowTitle.Trim().Length -gt 0 } |
  Select-Object Id, ProcessName, MainWindowTitle, @{Name='WindowId';Expression={$_.MainWindowHandle.ToString()}}
$items | ConvertTo-Json -Depth 4 -Compress
"#;
    let stdout = run_powershell(script).await?;
    let mut windows = match serde_json::from_str::<Value>(&stdout) {
        Ok(Value::Array(values)) => values,
        Ok(value) if !value.is_null() => vec![value],
        Ok(_) => Vec::new(),
        Err(_) => Vec::new(),
    };

    if !query.is_empty() {
        let query = query.to_ascii_lowercase();
        windows.retain(|window| {
            let title = window
                .get("MainWindowTitle")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_ascii_lowercase();
            let app = window
                .get("ProcessName")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_ascii_lowercase();
            title.contains(&query) || app.contains(&query)
        });
    }

    if !policy.window_allowlist.is_empty() {
        windows.retain(|window| {
            let title = window.get("MainWindowTitle").and_then(Value::as_str);
            let app = window.get("ProcessName").and_then(Value::as_str);
            enforce_window_allowlist(policy, title, app).is_ok()
        });
    }

    Ok(json!({
        "action": "window_list",
        "windows": windows,
        "count": windows.len(),
    }))
}

async fn handle_screen_capture(params: &Value) -> anyhow::Result<Value> {
    let output_path = optional_string(params, "path")
        .unwrap_or(r"C:\Temp\topclaw_capture.png")
        .to_string();
    let escaped_output_path = escape_ps_single_quoted(&output_path);
    let script = format!(
        r#"
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
$bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$bitmap = New-Object System.Drawing.Bitmap $bounds.Width, $bounds.Height
$graphics = [System.Drawing.Graphics]::FromImage($bitmap)
$graphics.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
$bitmap.Save('{escaped_output_path}', [System.Drawing.Imaging.ImageFormat]::Png)
$graphics.Dispose()
$bitmap.Dispose()
'{escaped_output_path}'
"#
    );
    run_powershell(&script).await?;
    Ok(json!({
        "action": "screen_capture",
        "path": output_path,
        "captured": true,
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
            native_input::mouse_move(x, y)?;
            Ok(json!({ "action": action, "x": x, "y": y, "accepted": true }))
        }
        "mouse_click" => {
            let x = enforce_coordinate_limit(params, "x", policy.max_coordinate_x)?
                .ok_or_else(|| anyhow::anyhow!("'x' is required"))?;
            let y = enforce_coordinate_limit(params, "y", policy.max_coordinate_y)?
                .ok_or_else(|| anyhow::anyhow!("'y' is required"))?;
            let button = optional_string(params, "button").unwrap_or("left");
            native_input::mouse_click(x, y, button)?;
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
            native_input::mouse_drag(from_x, from_y, to_x, to_y)?;
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
            native_input::key_type(text)?;
            Ok(json!({ "action": action, "text": text, "accepted": true }))
        }
        "key_press" => {
            let key = require_string(params, "key")?;
            native_input::key_press(key)?;
            Ok(json!({ "action": action, "key": key, "accepted": true }))
        }
        _ => anyhow::bail!("unsupported keyboard action '{action}'"),
    }
}

async fn handle_window_focus(params: &Value, policy: &ComputerUsePolicy) -> anyhow::Result<Value> {
    let target = extract_window_target(params, "window_focus")?;
    enforce_window_allowlist(
        policy,
        target.window_title.as_deref(),
        target.app.as_deref(),
    )?;
    let selector = build_window_selector_script(&target);
    let script = format!(
        r#"
{selector}
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class TopClawWindowNative {{
  [DllImport("user32.dll")]
  public static extern bool ShowWindowAsync(IntPtr hWnd, int nCmdShow);
  [DllImport("user32.dll")]
  public static extern bool SetForegroundWindow(IntPtr hWnd);
}}
"@
[void][TopClawWindowNative]::ShowWindowAsync($proc.MainWindowHandle, 9)
if (-not [TopClawWindowNative]::SetForegroundWindow($proc.MainWindowHandle)) {{
  $shell = New-Object -ComObject WScript.Shell
  if (-not $shell.AppActivate($proc.Id)) {{ throw 'failed to focus window' }}
}}
$proc | Select-Object Id, ProcessName, MainWindowTitle, @{{Name='WindowId';Expression={{$_.MainWindowHandle.ToString()}}}} | ConvertTo-Json -Depth 4 -Compress
"#
    );
    let stdout = run_powershell(&script).await?;
    let window = serde_json::from_str::<Value>(&stdout).unwrap_or_else(|_| {
        json!({
            "Id": Value::Null,
            "ProcessName": target.app,
            "MainWindowTitle": target.window_title,
            "WindowId": target.window_id,
        })
    });
    Ok(json!({
        "action": "window_focus",
        "accepted": true,
        "window": window,
    }))
}

async fn handle_window_close(params: &Value, policy: &ComputerUsePolicy) -> anyhow::Result<Value> {
    let target = extract_window_target(params, "window_close")?;
    enforce_window_allowlist(
        policy,
        target.window_title.as_deref(),
        target.app.as_deref(),
    )?;
    let selector = build_window_selector_script(&target);
    let script = format!(
        r#"
{selector}
$result = $proc.CloseMainWindow()
if (-not $result) {{ throw 'failed to close window' }}
$proc | Select-Object Id, ProcessName, MainWindowTitle, @{{Name='WindowId';Expression={{$_.MainWindowHandle.ToString()}}}} | ConvertTo-Json -Depth 4 -Compress
"#
    );
    let stdout = run_powershell(&script).await?;
    let window = serde_json::from_str::<Value>(&stdout).unwrap_or_else(|_| {
        json!({
            "Id": Value::Null,
            "ProcessName": target.app,
            "MainWindowTitle": target.window_title,
            "WindowId": target.window_id,
        })
    });
    Ok(json!({
        "action": "window_close",
        "accepted": true,
        "window": window,
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
        let script = format!("Stop-Process -Id {pid} -Force");
        run_powershell(&script).await?;
        return Ok(json!({
            "action": "app_terminate",
            "pid": pid,
            "accepted": true,
        }));
    }

    let app = require_string(params, "app")?;
    enforce_window_allowlist(policy, None, Some(app))?;
    let escaped = escape_ps_single_quoted(app);
    run_powershell(&format!("Stop-Process -Name '{escaped}' -Force")).await?;
    Ok(json!({
        "action": "app_terminate",
        "app": app,
        "accepted": true,
    }))
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

#[cfg(target_os = "windows")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let bind = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8789".to_string());
    let addr: SocketAddr = bind.parse()?;
    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/actions", post(handle_action))
        .with_state(AppState);

    println!("topclaw computer-use windows sidecar listening on http://{addr}");
    println!("health: http://{addr}/health");
    println!("actions: http://{addr}/v1/actions");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("computer_use_sidecar_windows can only run on Windows");
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
    fn extract_window_target_requires_selector_fields() {
        let err = extract_window_target(&json!({}), "window_focus").unwrap_err();
        assert!(err.to_string().contains("requires one of"));
    }

    #[test]
    fn build_window_selector_script_escapes_quotes() {
        let target = WindowTarget {
            window_id: None,
            window_title: Some("O'Hare".to_string()),
            app: Some("code".to_string()),
        };
        let script = build_window_selector_script(&target);
        assert!(script.contains("*O''Hare*"));
        assert!(script.contains("*code*"));
    }

    #[test]
    fn mouse_button_flag_maps_known_buttons() {
        assert_eq!(mouse_button_flag("left", true).unwrap(), 0x0002);
        assert_eq!(mouse_button_flag("right", false).unwrap(), 0x0010);
        assert!(mouse_button_flag("extra", true).is_err());
    }

    #[test]
    fn named_virtual_key_maps_common_keys() {
        assert_eq!(named_virtual_key("enter"), Some(0x0D));
        assert_eq!(named_virtual_key("ArrowLeft"), Some(0x25));
        assert_eq!(named_virtual_key("f12"), Some(0x7B));
        assert_eq!(named_virtual_key("unknown"), None);
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
}
