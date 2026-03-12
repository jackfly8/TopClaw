# Tài liệu Bắt đầu

Dành cho cài đặt lần đầu và làm quen nhanh.

## Lộ trình bắt đầu

1. Tổng quan và khởi động nhanh: [docs/i18n/vi/README.md](../README.md)
2. Cài đặt một lệnh và chế độ bootstrap kép: [../one-click-bootstrap.md](../one-click-bootstrap.md)
3. Hiểu mô hình runtime: [../runtime-model.md](../runtime-model.md)
4. Tìm lệnh theo tác vụ: [../commands-reference.md](../commands-reference.md)

## Chọn hướng đi

| Tình huống | Lệnh |
|----------|---------|
| Có API key, muốn cài nhanh nhất | `topclaw bootstrap --api-key sk-... --provider openrouter` |
| Dùng provider OAuth hoặc subscription thay vì API key | Chạy `topclaw bootstrap --interactive` và làm theo bước đăng nhập |
| Muốn được hướng dẫn từng bước | `topclaw bootstrap --interactive` |
| Đã có config, chỉ cần sửa kênh | `topclaw bootstrap --channels-only` |
| Chưa rõ nên dùng `agent`, `service` hay `daemon` | Xem [runtime-model.md](../runtime-model.md) |
| Dùng xác thực subscription | Xem [Subscription Auth](../../../README.md#subscription-auth-openai-codex--claude-code) |

## Thiết lập và kiểm tra

- Thiết lập nhanh: `topclaw bootstrap --api-key "sk-..." --provider openrouter`
- Thiết lập tương tác: `topclaw bootstrap --interactive`
- Kiểm tra môi trường: `topclaw status` + `topclaw status --diagnose`

## Tiếp theo

- Vận hành runtime: [../operations/README.md](../operations/README.md)
- Tra cứu tham khảo: [../reference/README.md](../reference/README.md)
