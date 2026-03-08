# Computer Use Sidecar Protocol

This document defines the TopClaw sidecar contract for `browser.backend = "computer_use"`.

The goal is cross-platform parity:

- macOS sidecars
- Windows sidecars
- Linux sidecars

All of them should accept the same TopClaw request envelope and return the same
response envelope, even if the local implementation uses very different native
APIs under the hood.

## Endpoint

Default endpoint:

```text
POST http://127.0.0.1:8787/v1/actions
```

Health probe:

```text
GET http://127.0.0.1:8787/health
```

If `browser.computer_use.api_key` is set, TopClaw sends:

```text
Authorization: Bearer <token>
```

## Request Envelope

```json
{
  "action": "window_focus",
  "params": {
    "window_title": "Visual Studio Code"
  },
  "policy": {
    "allowed_domains": ["example.com"],
    "window_allowlist": ["Visual Studio Code", "Google Chrome"],
    "max_coordinate_x": 3840,
    "max_coordinate_y": 2160
  },
  "metadata": {
    "session_name": "desktop-demo",
    "source": "topclaw.browser",
    "version": "0.1.2",
    "platform": "linux"
  }
}
```

Notes:

- `params` is action-specific.
- For `screen_capture`, TopClaw resolves `path` through its own file-write policy before sending the absolute path to the sidecar.
- Sidecars should reject unsupported actions explicitly instead of silently ignoring them.

## Response Envelope

Success:

```json
{
  "success": true,
  "data": {
    "ok": true,
    "action": "window_focus"
  }
}
```

Failure:

```json
{
  "success": false,
  "error": "window not found"
}
```

## Current Action Surface

Pointer / keyboard:

- `mouse_move`
- `mouse_click`
- `mouse_drag`
- `key_type`
- `key_press`
- `screen_capture`

Desktop / application control:

- `window_list`
- `window_focus`
- `window_close`
- `app_launch`
- `app_terminate`

Browser-aware:

- `open`

## Parameter Expectations

`mouse_move` / `mouse_click`

```json
{ "x": 640, "y": 480 }
```

`mouse_drag`

```json
{ "from_x": 200, "from_y": 300, "to_x": 600, "to_y": 300 }
```

`key_type`

```json
{ "text": "hello world" }
```

`key_press`

```json
{ "key": "Enter" }
```

`screen_capture`

```json
{ "path": "/tmp/topclaw_capture.png" }
```

`window_list`

```json
{ "query": "chrome" }
```

`window_focus` / `window_close`

At least one of:

- `window_id`
- `window_title`
- `app`

Example:

```json
{ "window_title": "Google Chrome" }
```

`app_launch`

```json
{ "app": "code", "args": ["--new-window"] }
```

`app_terminate`

At least one of:

- `app`
- `pid`

Example:

```json
{ "pid": 12345 }
```

## Platform Mapping Guidance

TopClaw does not require a single native implementation strategy.

Suggested platform mappings:

- macOS:
  - Accessibility API
  - CGEvent input synthesis
  - CoreGraphics screenshots
- Windows:
  - UI Automation
  - `SendInput`
  - GDI / Desktop Duplication screenshots
- Linux:
  - X11 backends
  - Wayland-compatible backends where available
  - desktop-environment-specific screenshot/input integrations as needed

## Security Expectations

Sidecars should:

- respect `window_allowlist`
- reject coordinates outside configured limits
- never expand domain or window scope on their own
- avoid implicit privilege escalation
- return explicit errors instead of guessing

## Reference Stub

TopClaw includes a protocol-compatible stub server for development:

```bash
cargo run --example computer_use_sidecar_stub
```

It does not perform real desktop automation. It exists to make the request and
response contract concrete for integrators building macOS, Windows, or Linux
sidecars.

## Linux Reference Sidecar

TopClaw also includes a Linux-first example:

```bash
cargo run --example computer_use_sidecar_linux
```

Scope and limitations:

- uses common Linux desktop binaries such as `xdotool`, `wmctrl`, `xdg-open`, `gnome-screenshot`, `scrot`, and `pkill`
- enforces `allowed_domains`, `window_allowlist`, and coordinate limits from the TopClaw policy envelope
- treats pointer, keyboard, and window-control actions as X11-only
- does not claim full Wayland parity; on Wayland it fails fast for unsupported X11-style actions

## Windows Reference Sidecar

TopClaw also includes a Windows-first example:

```bash
cargo run --example computer_use_sidecar_windows
```

Scope and limitations:

- implements the full current TopClaw action surface, including mouse and keyboard input
- uses PowerShell and .NET helpers for screenshot capture and process/window enumeration
- uses `user32.dll` for mouse movement, mouse clicks/drags, key typing, and key presses
- enforces `allowed_domains` and `window_allowlist` from the TopClaw policy envelope
- key typing uses Unicode `SendInput`, while named special keys still map through Windows virtual-key handling

Operations guide:

- [operations/computer-use-sidecar-runbook.md](operations/computer-use-sidecar-runbook.md)

## macOS Reference Sidecar

TopClaw also includes a macOS-first example:

```bash
cargo run --example computer_use_sidecar_macos
```

Scope and limitations:

- implements the full current TopClaw action surface
- uses `open`, `screencapture`, and `osascript` for application, window, keyboard, and screenshot actions
- uses `cliclick` for mouse movement, mouse clicks, and mouse drags
- enforces `allowed_domains`, `window_allowlist`, and coordinate limits from the TopClaw policy envelope
- requires Accessibility permissions for `osascript` / System Events, and `cliclick` must be installed for mouse actions
