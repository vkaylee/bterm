# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- **PTY:** Triển khai cơ chế thay đổi kích thước terminal (dynamic resizing) thông qua phương thức `MasterPty::resize`.
- **UI:** Nâng cấp cấu hình xterm.js với `letterSpacing`, `lineHeight` và theme tối ưu hơn để tăng chất lượng hiển thị.

### Fixed
- **UI:** Khắc phục lỗi terminal bị giới hạn chiều ngang 50% bằng cách đồng bộ hóa kích thước PTY với giao diện người dùng.
- **Testing:** Sửa lỗi các bài kiểm tra E2E do thiếu tham chiếu terminal trên đối tượng `window`.
