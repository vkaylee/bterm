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
   - Hàm `monitor_session` lắng nghe channel này để:
     - Cập nhật buffer lịch sử (100KB gần nhất).
     - Phát hiện **Termination Signal** (vector rỗng) và tự động xóa session khỏi `SessionRegistry`.
   - `ws_handler` nhận tín hiệu này qua channel, thoát vòng lặp binary và gửi gói tin JSON `{"type": "Exit"}` tới Browser trước khi đóng socket.

2. **Client Input -> PTY:**
   - Browser gửi WebSocket message dạng JSON (`Input`).
   - Server trích xuất `data` và ghi trực tiếp vào PTY Master Writer.

3. **Terminal Resizing:**
   - Khi kích thước cửa sổ trình duyệt thay đổi, `ResizeObserver` kích hoạt `FitAddon`.
   - Browser gửi WebSocket message dạng JSON (`Resize`) chứa số hàng (`rows`) và cột (`cols`) mới.
   - `PtyManager` sử dụng `MasterPty::resize` để cập nhật kích thước của tiến trình shell đang chạy, đảm bảo dữ liệu hiển thị không bị vỡ hoặc giới hạn ở 80 cột.

## Tối ưu hóa cho Mobile
Để đảm bảo trải nghiệm tốt trên thiết bị di động, BTerminal thực hiện các kỹ thuật sau:
- **Viewport Management:** Tự động điều chỉnh kích thước PTY khi bàn phím ảo xuất hiện/biến mất thông qua `VisualViewport API`. Thay vì chỉ dùng `ResizeObserver`, ứng dụng lắng nghe sự kiện `resize` của viewport thực tế để đẩy thanh phím ảo lên trên bàn phím hệ thống bằng `translateY`.
- **Sticky Modifiers:** Giải quyết vấn đề gõ tổ hợp phím trên mobile. Khi nhấn Ctrl/Alt trên màn hình, ứng dụng sẽ chuyển sang trạng thái "active" và đợi phím gõ tiếp theo từ bàn phím hệ thống (vd: nhấn Ctrl rồi gõ 'c' sẽ gửi mã byte `\x03`).
- **Input Focus Preservation:** Sử dụng `event.preventDefault()` trên sự kiện `onmousedown` của các nút ảo để ngăn trình duyệt chuyển focus khỏi terminal, giúp bàn phím hệ thống luôn mở khi người dùng thao tác với phím bổ trợ.
- **Render Refresh:** Ép buộc Xterm.js thực hiện render cycle (`term.refresh()`) và cuộn xuống cuối (`term.scrollToBottom()`) khi nhận dữ liệu mới để tránh lỗi màn hình đen trên một số trình duyệt di động.

## Deployment
- Ứng dụng được đóng gói dưới dạng **Single Binary**.
- Frontend (HTML/JS/CSS) và toàn bộ tài nguyên (Fonts, Libraries) được nhúng trực tiếp vào binary Rust bằng `rust-embed`.
- Dự án không yêu cầu kết nối Internet khi khởi chạy (Zero external dependencies at runtime).

## Testing Architecture

BTerminal employs a strictly isolated testing strategy to ensure reliability and parallelism:

1.  **Unit & Integration Tests**: Backend logic (PTY management, session monitoring) is tested using standard `cargo test`. Integration tests verify the dynamic port binding logic by simulating collisions and environment overrides.
2.  **E2E Isolation (Worker-Scoped)**: 
    - Playwright is configured to run tests in parallel.
    - Each test worker utilizes a custom fixture (`e2e/fixtures.ts`) that spawns a dedicated BTerminal backend instance.
    - Servers are bound to a random port (`PORT=0`) provided by the OS, preventing conflict between parallel workers.
    - This architecture ensures that each test suite starts with a fresh `SessionRegistry`, preventing state leakage and making tests deterministic.
