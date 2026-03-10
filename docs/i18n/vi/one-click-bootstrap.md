# Cài đặt một lệnh

Cách cài đặt và khởi tạo TopClaw nhanh nhất.

Xác minh lần cuối: **2026-03-09**.

## Cập nhật an toàn

Với bản cài đặt đang dùng, cách cập nhật nhanh và được hỗ trợ là:

```bash
topclaw update
topclaw --version
```

Chỉ kiểm tra trước mà không cài:

```bash
topclaw update --check
```

Nếu TopClaw đang chạy như dịch vụ nền, hãy khởi động lại dịch vụ sau khi cập nhật:

```bash
topclaw service restart
```

Nếu `topclaw update` báo không thể ghi đè binary hiện tại, hãy quay về đúng phương thức cài đặt ban đầu:

- cài từ repo checkout: `./bootstrap.sh --prefer-prebuilt`
- cài từ source: `cargo install --path . --force --locked`
- cài qua package manager: cập nhật bằng chính package manager đó

## Cách 0: Homebrew (macOS/Linuxbrew)

```bash
brew install topclaw
```

## Cách A (Khuyến nghị): Clone + chạy script cục bộ

```bash
git clone https://github.com/topway-ai/topclaw.git
cd topclaw
./bootstrap.sh --install-system-deps --install-rust --prefer-prebuilt
```

Đường dẫn khuyến nghị này sẽ:

1. cài các phụ thuộc hệ thống tiêu chuẩn khi được hỗ trợ
2. cài Rust nếu chưa có
3. ưu tiên binary dựng sẵn trước
4. chỉ fallback sang build từ source khi không có release asset tương thích

### Kiểm tra tài nguyên và binary dựng sẵn

Build từ mã nguồn thường yêu cầu tối thiểu:

- **2 GB RAM + swap**
- **6 GB dung lượng trống**

Khi tài nguyên hạn chế, bootstrap sẽ thử tải binary dựng sẵn trước.

```bash
./bootstrap.sh --prefer-prebuilt
```

Chỉ dùng binary dựng sẵn, báo lỗi nếu không tìm thấy bản phù hợp:

```bash
./bootstrap.sh --prebuilt-only
```

Bỏ qua binary dựng sẵn, buộc build từ mã nguồn:

```bash
./bootstrap.sh --force-source-build
```

## Bootstrap kép

Mặc định là **chỉ ứng dụng** (build/cài TopClaw), yêu cầu Rust toolchain sẵn có.

Với máy mới, bật bootstrap môi trường:

```bash
./bootstrap.sh --install-system-deps --install-rust
```

Lưu ý:

- `--install-system-deps` cài các thành phần biên dịch/build cần thiết (có thể cần `sudo`).
- `--install-rust` cài Rust qua `rustup` nếu chưa có.
- `--prefer-prebuilt` thử tải binary dựng sẵn trước, nếu không có thì build từ nguồn.
- `--prebuilt-only` tắt phương án build từ nguồn.
- `--force-source-build` tắt hoàn toàn phương án binary dựng sẵn.
- `--prefer-prebuilt` có thể cài binary của bản phát hành mới nhất thay vì build chính checkout hiện tại; dùng `./bootstrap.sh --force-source-build` khi cần kiểm tra thay đổi mã nguồn cục bộ.

## Cách B: Lệnh từ xa một dòng

```bash
curl -fsSL https://raw.githubusercontent.com/topway-ai/topclaw/main/scripts/bootstrap.sh | bash
```

Với môi trường yêu cầu bảo mật cao, nên dùng Cách A để kiểm tra script trước khi chạy.

Tương thích ngược:

```bash
curl -fsSL https://raw.githubusercontent.com/topway-ai/topclaw/main/scripts/install.sh | bash
```

Endpoint cũ này ưu tiên chuyển tiếp đến `scripts/bootstrap.sh`, nếu không có thì dùng cài đặt từ nguồn kiểu cũ.

Nếu chạy Cách B ngoài thư mục repo, bootstrap script sẽ tự clone workspace tạm, build, cài đặt rồi dọn dẹp.

## Chế độ thiết lập tùy chọn

### Thiết lập trong container (Docker)

```bash
./bootstrap.sh --docker
```

Lệnh này build image TopClaw cục bộ và chạy thiết lập trong container, lưu config/workspace vào `./.topclaw-docker`.

CLI container mặc định là `docker`. Nếu Docker CLI không có mà `podman` tồn tại, bootstrap sẽ tự fallback sang `podman`. Bạn cũng có thể đặt `TOPCLAW_CONTAINER_CLI` tường minh (ví dụ: `TOPCLAW_CONTAINER_CLI=podman ./bootstrap.sh --docker`).

Với Podman, bootstrap chạy cùng `--userns keep-id` và volume label `:Z` để mount workspace/config vẫn ghi được trong container.

Nếu thêm `--skip-build`, bootstrap sẽ bỏ qua bước build image cục bộ. Nó sẽ thử Docker tag cục bộ trước (`TOPCLAW_DOCKER_IMAGE`, mặc định: `topclaw-bootstrap:local`); nếu không có thì kéo `ghcr.io/topway-ai/topclaw:latest` rồi tag lại cục bộ trước khi chạy.

### Thiết lập nhanh (không tương tác)

```bash
./bootstrap.sh --onboard --api-key "sk-..." --provider openrouter
```

Hoặc dùng biến môi trường:

```bash
TOPCLAW_API_KEY="sk-..." TOPCLAW_PROVIDER="openrouter" ./bootstrap.sh --onboard
```

### Thiết lập tương tác

```bash
./bootstrap.sh --interactive-onboard
```

## Các cờ hữu ích

- `--install-system-deps`
- `--install-rust`
- `--skip-build` (trong chế độ `--docker`: dùng image cục bộ nếu có, nếu không sẽ kéo `ghcr.io/topway-ai/topclaw:latest`)
- `--skip-install`
- `--provider <id>`

Xem tất cả tùy chọn:

```bash
./bootstrap.sh --help
```

## Tài liệu liên quan

- [docs/i18n/vi/README.md](README.md)
- [commands-reference.md](commands-reference.md)
- [providers-reference.md](providers-reference.md)
- [channels-reference.md](channels-reference.md)
