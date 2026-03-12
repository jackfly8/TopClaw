# Khắc phục sự cố TopClaw (Tiếng Việt)

Đây là trang cầu nối cho các lỗi cài đặt, runtime và channel thường gặp.

Nguồn tiếng Anh:

- [../../troubleshooting.md](../../troubleshooting.md)

## Dùng khi nào

- TopClaw không phản hồi sau onboarding
- Provider auth bị thiếu hoặc hết hạn
- Service hoặc channel chạy nền không hoạt động như mong đợi

## Quy ước

- Mã lỗi, khóa log và chữ ký lỗi chuẩn lấy theo bản tiếng Anh.
- Để chẩn đoán theo thứ tự an toàn, ưu tiên:
  `topclaw status` -> `topclaw status --diagnose` -> `topclaw service status` -> `topclaw channel doctor`
