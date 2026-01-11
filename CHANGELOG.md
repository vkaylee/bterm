# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- **PTY:** Triển khai cơ chế thay đổi kích thước terminal (dynamic resizing) thông qua phương thức `MasterPty::resize`.
- **UI:** Nâng cấp cấu hình xterm.js với `letterSpacing`, `lineHeight` và theme tối ưu hơn để tăng chất lượng hiển thị.
- **Mobile UI:** Thiết kế lại thanh điều khiển di động thành 2 hàng (7 cột mỗi hàng) tối ưu diện tích.
- **Mobile UI:** Triển khai cơ chế **Sticky Modifiers** (Ctrl/Alt dính) cho phép thực hiện các tổ hợp phím (như Ctrl+C) dễ dàng trên điện thoại.

### Fixed
- **UI:** Khắc phục lỗi terminal bị giới hạn chiều ngang 50% bằng cách đồng bộ hóa kích thước PTY với giao diện người dùng.
- **UI:** Sửa lỗi bàn phím di động bị đóng sau khi nhấn phím ảo bằng cách xử lý sự kiện `onmousedown`.
- **UI:** Giải quyết vấn đề thanh điều khiển bị bàn phím che lấp bằng VisualViewport API.
- **Testing:** Sửa lỗi các bài kiểm tra E2E do thiếu tham chiếu terminal trên đối tượng `window`.
