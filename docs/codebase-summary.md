# Codebase Summary

## Backend (Rust)
- `src/lib.rs`: The core library containing application logic, routing, and shared modules.
- `src/main.rs`: Thin binary wrapper. Sets up the Axum server with dynamic port selection (Env > 3000 > Random) and serves the app.
- `src/api.rs`: REST API endpoints for session management (List, Create, Delete).
- `src/ws.rs`: WebSocket handler for terminal I/O, PTY resizing, and session history synchronization.
- **`src/session.rs`**: Session lifecycle management. Includes `SessionRegistry` (now integrated with global broadcast) and `monitor_session` for resource cleanup and real-time deletion notifications.
- **`src/pty_manager.rs`**: Direct OS interface for PTY creation and control.

## Frontend (HTML/JS)
- `frontend/dist/index.html`: Giao diện người dùng duy nhất (SPA). Sử dụng Tailwind CSS cho layout và Xterm.js cho terminal.
- **Tính năng nổi bật:**
  - **3-Tier Rendering Engine**: Hệ thống tự động chọn renderer tối ưu nhất theo thứ tự: **WebGL** (60fps performance) -> **Canvas** (Stable high-perf) -> **DOM** (Fallback).
  - **Adaptive Font Size:** Tự động điều chỉnh cỡ chữ terminal (15px cho desktop, 14px cho mobile).
  - **Robust Layout Engine:** Sử dụng `@xterm/addon-fit` chính chủ với cơ chế retry thông minh để đảm bảo terminal luôn full width/height ngay cả khi renderer khởi động chậm.
  - **Dynamic Viewport Scaling:** Sử dụng `VisualViewport API` để thay đổi chiều cao ứng dụng khi bàn phím mobile hiện lên, ngăn chặn việc che khuất nội dung.
  - **Sticky Modifiers:** Hỗ trợ phím chức năng ảo (Ctrl, Alt) trên mobile.
  - **Real-time Sync:** Tự động cập nhật danh sách session qua SSE.
  - **Single Binary:** Tích hợp sâu với backend qua `rust-embed`.

## Testing & Quality
- **Coverage**: **95.12% line coverage** achieved for the Rust backend.
- `tests/sse_integration.rs`: New integration tests for SSE event streaming.
- `e2e/tests/sync-management.spec.ts`: Verification of multi-device synchronization.
- `e2e/tests/terminal-layout.spec.ts`: Verification of full-screen layout and resizing.
- `e2e/tests/terminal-advanced.spec.ts`: **New** Advanced verification for PTY resizing, special keys (Ctrl+C), session persistence, and UTF-8 rendering.

