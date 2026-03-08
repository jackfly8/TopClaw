# Computer Use Sidecar Runbook

Use this runbook when operating TopClaw with `browser.backend = "computer_use"`.

## Safety First

- Keep the sidecar bound to loopback only.
- Use a dedicated test machine or VM when possible.
- Restrict `browser.allowed_domains` and `browser.computer_use.window_allowlist`.
- Set `max_coordinate_x` and `max_coordinate_y` so accidental clicks stay bounded.

## Linux

Install common helper tools:

```bash
sudo apt-get install -y xdotool wmctrl xdg-utils scrot
```

Start the Linux sidecar:

```bash
cargo run --example computer_use_sidecar_linux
```

Recommended TopClaw config:

```toml
[browser]
enabled = true
backend = "computer_use"
allowed_domains = ["github.com", "docs.topclaw.dev"]
session_name = "linux-desktop"

[browser.computer_use]
endpoint = "http://127.0.0.1:8788/v1/actions"
allow_remote_endpoint = false
window_allowlist = ["Visual Studio Code", "Firefox", "Chromium"]
max_coordinate_x = 3840
max_coordinate_y = 2160
```

Operational notes:

- Pointer, keyboard, and window-control actions are X11-first.
- On Wayland, unsupported X11-style actions fail explicitly.
- Screenshot capture falls back across `gnome-screenshot`, `scrot`, and ImageMagick `import`.

## Windows

Prerequisites:

- PowerShell available on `PATH`
- Desktop session unlocked
- TopClaw built on a Windows host

Start the Windows sidecar:

```powershell
cargo run --example computer_use_sidecar_windows
```

Recommended TopClaw config:

```toml
[browser]
enabled = true
backend = "computer_use"
allowed_domains = ["github.com", "docs.topclaw.dev"]
session_name = "windows-desktop"

[browser.computer_use]
endpoint = "http://127.0.0.1:8789/v1/actions"
allow_remote_endpoint = false
window_allowlist = ["Visual Studio Code", "Chrome", "Firefox", "Notepad"]
max_coordinate_x = 3840
max_coordinate_y = 2160
```

Operational notes:

- Screenshots and window enumeration use PowerShell/.NET helpers.
- Mouse and keyboard input use `user32.dll`.
- Typed text now uses Unicode `SendInput`.
- Window focus/close depend on a normal interactive desktop session and may fail on locked or service-only sessions.

## macOS

Prerequisites:

- `osascript` and `screencapture` available on the host
- Accessibility permissions granted to Terminal or the shell launching the sidecar
- `cliclick` installed for mouse actions:

```bash
brew install cliclick
```

Start the macOS sidecar:

```bash
cargo run --example computer_use_sidecar_macos
```

Recommended TopClaw config:

```toml
[browser]
enabled = true
backend = "computer_use"
allowed_domains = ["github.com", "docs.topclaw.dev"]
session_name = "macos-desktop"

[browser.computer_use]
endpoint = "http://127.0.0.1:8790/v1/actions"
allow_remote_endpoint = false
window_allowlist = ["Visual Studio Code", "Safari", "Google Chrome", "Terminal"]
max_coordinate_x = 3456
max_coordinate_y = 2234
```

Operational notes:

- Mouse actions depend on `cliclick`.
- Keyboard, window enumeration, focus, and close depend on `osascript` with System Events access.
- Screenshot capture uses the built-in `screencapture` command.
- If macOS privacy prompts block automation, re-grant Accessibility permissions and restart the sidecar.

## Smoke Checks

Check health:

```bash
curl http://127.0.0.1:8788/health
```

Windows PowerShell equivalent:

```powershell
Invoke-WebRequest http://127.0.0.1:8789/health | Select-Object -Expand Content
```

macOS health check:

```bash
curl http://127.0.0.1:8790/health
```

Basic action test:

```bash
curl -sS http://127.0.0.1:8788/v1/actions \
  -H 'content-type: application/json' \
  -d '{"action":"window_list","params":{},"policy":{"window_allowlist":["Visual Studio Code"]},"metadata":{"source":"manual"}}'
```

## Rollback

If computer use behaves unexpectedly:

1. Stop the sidecar process.
2. Set `[browser].backend` back to `agent_browser` or disable `[browser].enabled`.
3. Remove or narrow `window_allowlist`.
4. Re-run `topclaw doctor` before re-enabling the sidecar.
