# Runbook sidecar computer-use

Dùng runbook này khi vận hành TopClaw với `browser.backend = "computer_use"`.

## An toàn trước tiên

- Chỉ bind sidecar vào loopback.
- Ưu tiên máy thử nghiệm hoặc VM riêng.
- Giới hạn `browser.allowed_domains` và `browser.computer_use.window_allowlist`.
- Đặt `max_coordinate_x` và `max_coordinate_y` để giới hạn vùng click.

## Linux

Cài các công cụ phụ trợ phổ biến:

```bash
sudo apt-get install -y xdotool wmctrl xdg-utils scrot
```

Khởi động sidecar Linux:

```bash
cargo run --example computer_use_sidecar_linux
```

Cấu hình TopClaw khuyến nghị:

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

Lưu ý vận hành:

- Các hành động chuột, bàn phím và cửa sổ hiện theo hướng X11-first.
- Trên Wayland, các hành động kiểu X11 không được hỗ trợ sẽ fail rõ ràng.
- Chụp màn hình fallback qua `gnome-screenshot`, `scrot` và ImageMagick `import`.

## Windows

Điều kiện cần:

- Có PowerShell trên `PATH`
- Phiên desktop đang mở khóa
- Build TopClaw trên máy Windows

Khởi động sidecar Windows:

```powershell
cargo run --example computer_use_sidecar_windows
```

Cấu hình TopClaw khuyến nghị:

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

Lưu ý vận hành:

- Screenshot và liệt kê cửa sổ dùng PowerShell/.NET.
- Chuột và bàn phím dùng `user32.dll`.
- Gõ văn bản hiện dùng Unicode `SendInput`.
- Focus/close cửa sổ phụ thuộc phiên desktop tương tác bình thường; có thể thất bại trên session bị khóa hoặc chỉ chạy dạng service.

## macOS

Điều kiện cần:

- Có `osascript` và `screencapture` trên máy
- Cấp quyền Accessibility cho Terminal hoặc shell chạy sidecar
- Cài `cliclick` cho các hành động chuột:

```bash
brew install cliclick
```

Khởi động sidecar macOS:

```bash
cargo run --example computer_use_sidecar_macos
```

Cấu hình TopClaw khuyến nghị:

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

Lưu ý vận hành:

- Các hành động chuột phụ thuộc `cliclick`.
- Bàn phím, liệt kê/focus/close cửa sổ phụ thuộc `osascript` với quyền System Events.
- Chụp màn hình dùng lệnh tích hợp `screencapture`.
- Nếu prompt quyền riêng tư của macOS chặn tự động hóa, hãy cấp lại quyền Accessibility rồi khởi động lại sidecar.

## Kiểm tra nhanh

Kiểm tra health:

```bash
curl http://127.0.0.1:8788/health
```

Tương đương trong PowerShell:

```powershell
Invoke-WebRequest http://127.0.0.1:8789/health | Select-Object -Expand Content
```

Kiểm tra health cho macOS:

```bash
curl http://127.0.0.1:8790/health
```

Test hành động cơ bản:

```bash
curl -sS http://127.0.0.1:8788/v1/actions \
  -H 'content-type: application/json' \
  -d '{"action":"window_list","params":{},"policy":{"window_allowlist":["Visual Studio Code"]},"metadata":{"source":"manual"}}'
```

## Rollback

Nếu computer use hoạt động không như mong đợi:

1. Dừng process sidecar.
2. Đặt `[browser].backend` về `agent_browser` hoặc tắt `[browser].enabled`.
3. Gỡ hoặc thu hẹp `window_allowlist`.
4. Chạy lại `topclaw doctor` trước khi bật lại sidecar.
