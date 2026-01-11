# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- **UI:** Thiết kế lại Header chuyên nghiệp tích hợp thông tin Session ID, trạng thái kết nối (Pulse indicator) và logo mới.
- **UI:** Tái cấu trúc Dashboard theo hướng "Nội dung trước" (Content-first): đưa danh sách Active Sessions lên đầu và phần tạo mới xuống cuối.
- **UI:** Thêm bộ đếm Session hoạt động và giao diện "Empty State" sinh động khi không có session nào.
- **Mobile UI:** Thiết kế lại thanh điều khiển di động thành 2 hàng (7 cột mỗi hàng) tối ưu diện tích.
- **Mobile UI:** Triển khai cơ chế **Sticky Modifiers** (Ctrl/Alt dính) cho phép thực hiện các tổ hợp phím (như Ctrl+C) dễ dàng trên điện thoại.

### Fixed
- **UI:** Khắc phục lỗi terminal bị giới hạn chiều ngang 50% bằng cách đồng bộ hóa kích thước PTY với giao diện người dùng.
- **UI:** Sửa lỗi bàn phím di động bị đóng sau khi nhấn phím ảo bằng cách xử lý sự kiện `onmousedown`.
- **UI:** Giải quyết vấn đề thanh điều khiển bị bàn phím che lấp bằng VisualViewport API.
- **Testing:** Cập nhật các bộ test E2E để tương thích với cấu trúc Header và Dashboard mới.
