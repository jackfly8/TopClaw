# Mô hình runtime TopClaw

Trang này giải thích rõ khi nào nên dùng `topclaw agent`, `service`, `daemon`, `gateway` và `channel start`.

Xác minh lần cuối: **2026-03-09**.

## Tóm tắt nhanh

| Mục tiêu | Lệnh | Khi dùng |
|---|---|---|
| Trò chuyện trực tiếp trong terminal hiện tại | `topclaw agent` | thử nhanh, gửi prompt một lần |
| Giữ các channel đã cấu hình chạy nền | `topclaw service install`, `topclaw service start`, `topclaw service status` | Telegram, Discord và các kênh luôn bật |
| Chạy toàn bộ runtime ở foreground | `topclaw daemon` | gỡ lỗi khởi động và xem log trực tiếp |
| Chỉ chạy gateway HTTP/webhook | `topclaw gateway` | kiểm thử webhook hoặc gateway riêng |
| Tự chạy channel ở foreground | `topclaw channel start` | khắc phục sự cố nâng cao |

## Sau onboarding nên hiểu thế nào

- provider nên sẵn sàng, hoặc TopClaw sẽ chỉ rõ lệnh auth tiếp theo
- model mặc định đã được chọn
- channel đã chọn đã được ghi vào `config.toml`
- khi nền tảng hỗ trợ, onboarding sẽ cố gắng cài và khởi động service tự động cho các setup cần chạy nền

## Hỗ trợ service theo môi trường

| Môi trường | Kỳ vọng về service nền |
|---|---|
| Linux với `systemd --user` | onboarding thường tự quản lý nếu channel cần chạy nền |
| macOS với `launchd` | onboarding thường tự quản lý nếu channel cần chạy nền |
| Windows runtime được quản lý | onboarding thường tự quản lý nếu channel cần chạy nền |
| OpenRC hoặc môi trường thủ công khác | có thể vẫn phải chạy `topclaw service install` và `topclaw service start` bằng tay |

## Tôi nên chạy lệnh nào?

### Chỉ muốn thử ngay

```bash
topclaw agent -m "Hello, TopClaw!"
```

### Đã cấu hình Telegram/Discord hoặc channel chạy nền

```bash
topclaw service status
```

Nếu chưa chạy:

```bash
topclaw service install
topclaw service start
```

Nếu bạn đang ở môi trường chỉ hỗ trợ service thủ công, onboarding có thể dừng sau khi ghi config và yêu cầu bạn tự chạy các lệnh service.

### Muốn gỡ lỗi ở foreground

```bash
topclaw daemon
```

### Muốn tự chạy channel bằng tay

```bash
topclaw channel doctor
topclaw channel start
```

Với vận hành bình thường, ưu tiên `topclaw service ...` hơn `topclaw channel start`.
