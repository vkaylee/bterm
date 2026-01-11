# System Architecture

## Thành phần hệ thống

```ascii
┌──────────────┐      HTTP (JSON)      ┌──────────────────┐
│   Browser    │◄─────────────────────▶│   Axum Server    │
│  (Dashboard) │                       │ (SessionRegistry)│
└──────────────┘                       └────────┬─────────┘
        ▲                                       │
        │ WebSockets (Binary)                   │ Spawn
        ▼                                       ▼
┌──────────────┐                       ┌──────────────────┐
│   Xterm.js   │◄─────────────────────▶│   PTY Instance   │
│  (Terminal)  │    Broadcast Channel  │ (/bin/bash etc.) │
└──────────────┘                       └──────────────────┘
```

## Luồng dữ liệu (Data Flow)
1. **PTY Output -> Clients:**
   - PTY Process tạo output bytes.
   - Một luồng (thread) đọc output và gửi vào `tokio::sync::broadcast` channel.
   - `SessionRegistry` lắng nghe channel này để cập nhật buffer lịch sử (100KB gần nhất).
   - Tất cả WebSockets đang kết nối tới session đó nhận dữ liệu từ broadcast và gửi về Browser.

2. **Client Input -> PTY:**
   - Browser gửi WebSocket message dạng JSON (`Input`).
   - Server trích xuất `data` và ghi trực tiếp vào PTY Master Writer.

3. **Terminal Resizing:**
   - Khi kích thước cửa sổ trình duyệt thay đổi, `ResizeObserver` kích hoạt `FitAddon`.
   - Browser gửi WebSocket message dạng JSON (`Resize`) chứa số hàng (`rows`) và cột (`cols`) mới.
   - `PtyManager` sử dụng `MasterPty::resize` để cập nhật kích thước của tiến trình shell đang chạy, đảm bảo dữ liệu hiển thị không bị vỡ hoặc giới hạn ở 80 cột.

## Tối ưu hóa cho Mobile
Để đảm bảo trải nghiệm tốt trên thiết bị di động, BTerminal thực hiện các kỹ thuật sau:
- **Viewport Management:** Tự động điều chỉnh kích thước PTY khi bàn phím ảo xuất hiện/biến mất thông qua `ResizeObserver`.
- **Render Refresh:** Ép buộc Xterm.js thực hiện render cycle (`term.refresh()`) và cuộn xuống cuối (`term.scrollToBottom()`) khi nhận dữ liệu mới để tránh lỗi màn hình đen trên một số trình duyệt di động.
- **On-screen Controls:** Cung cấp các phím chức năng (Ctrl, Alt, Esc, Arrow Keys) phía dưới terminal view trên màn hình nhỏ.

## Deployment
- Ứng dụng được đóng gói dưới dạng **Single Binary**.
- Frontend (HTML/JS/CSS) và toàn bộ tài nguyên (Fonts, Libraries) được nhúng trực tiếp vào binary Rust bằng `rust-embed`.
- Dự án không yêu cầu kết nối Internet khi khởi chạy (Zero external dependencies at runtime).
