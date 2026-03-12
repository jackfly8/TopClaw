# Tài liệu Bắt đầu

Dành cho cài đặt lần đầu và làm quen nhanh.

## Lộ trình bắt đầu

1. Tổng quan và khởi động nhanh: [docs/i18n/vi/README.md](../README.md)
2. Cài đặt bằng lệnh một dòng phù hợp với shell của bạn:
   `curl -fsSL https://raw.githubusercontent.com/topway-ai/topclaw/main/scripts/bootstrap.sh | bash`
   hoặc PowerShell:
   `iwr -useb https://raw.githubusercontent.com/topway-ai/topclaw/main/bootstrap.ps1 | iex`
3. Xem chi tiết bootstrap và các cờ cài đặt: [../one-click-bootstrap.md](../one-click-bootstrap.md)
4. Hiểu mô hình runtime: [../runtime-model.md](../runtime-model.md)
5. Tìm lệnh theo tác vụ: [../commands-reference.md](../commands-reference.md)

## Chọn hướng đi

| Tình huống | Lệnh |
|----------|---------|
| Có API key, muốn cài nhanh nhất | `topclaw bootstrap --api-key sk-... --provider openrouter` |
| Dùng provider OAuth hoặc subscription thay vì API key | Chạy `topclaw bootstrap --interactive` và làm theo bước đăng nhập |
| Muốn được hướng dẫn từng bước | `topclaw bootstrap --interactive` |
| Đã có config, chỉ cần sửa kênh | `topclaw bootstrap --channels-only` |
| Chưa rõ nên dùng `agent`, `service` hay `daemon` | Xem [runtime-model.md](../runtime-model.md) |
| Dùng xác thực subscription | Xem [README.md](../../../README.md) để bắt đầu bằng luồng cài đặt và đăng nhập tương tác |

## Thiết lập và kiểm tra

- Thiết lập nhanh: `topclaw bootstrap --api-key "sk-..." --provider openrouter`
- Thiết lập tương tác: `topclaw bootstrap --interactive`
- Kiểm tra môi trường: `topclaw status` + `topclaw status --diagnose`

## Tiếp theo

- Vận hành runtime: [../operations/README.md](../operations/README.md)
- Tra cứu tham khảo: [../reference/README.md](../reference/README.md)
