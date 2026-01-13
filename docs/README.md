# BTerminal

**BTerminal** là một ứng dụng Web Terminal đa phiên (multi-session) được viết bằng Rust. Ứng dụng cho phép người dùng tạo, quản lý và chia sẻ các phiên làm việc terminal thông qua trình duyệt web với tính năng đồng bộ thời gian thực.

## Tính năng chính
- 🚀 **Single Binary:** Backend và Frontend được đóng gói thành một file thực thi duy nhất.
- 🛡️ **Security-First:** Bắt buộc đổi mật khẩu khi cài đặt lần đầu.
- 🔄 **Session Persistence:** Giữ phiên làm việc ngay cả khi đóng trình duyệt.
- 👥 **Shared Sessions:** Nhiều thiết bị có thể kết nối vào cùng một session và thấy kết quả giống nhau.
- 🗑️ **Session Management:** Dashboard cho phép tạo và xóa các phiên làm việc dễ dàng.
- ⚡ **High Performance:** Xây dựng trên nền tảng Rust, Axum và Tokio.

## Hướng dẫn chi tiết
- [Kiến trúc hệ thống](./system-architecture.md)
- [Tham chiếu API](./api-reference.md)
- [Hướng dẫn triển khai (Deployment)](./deployment-guide.md)
- [Tiêu chuẩn mã nguồn](./code-standards.md)

## Công nghệ sử dụng
- **Backend:** Rust, Axum (Web Server), Tokio (Async runtime), Portable-PTY.
- **Frontend:** Xterm.js, Tailwind CSS, Vanilla JS.
- **Communication:** WebSockets (Real-time), REST API (Management).

## Bắt đầu nhanh
```bash
# Biên dịch và chạy
cargo run

# Truy cập qua trình duyệt
http://localhost:3000
```
