# Codebase Summary

## Backend (Rust)
- `src/lib.rs`: The core library containing application logic, routing, and shared modules. Now includes **Authentication Middleware** and session cookie management.
- `src/main.rs`: Thin binary wrapper. Sets up the Axum server with dynamic port selection and initializes the **SQLite Database**.
- `src/api.rs`: REST API endpoints for session management (Protected by Auth).
- `src/auth.rs`: **New** Logic for user authentication, password hashing (Argon2), and session handlers (Login/Logout/Me).
- `src/db.rs`: **New** Database abstraction layer using **SQLite (SQLx)**. Handles user persistence and auto-migration.
- `src/ws.rs`: WebSocket handler for terminal I/O. Connections require a valid auth session. Now supports **Initial State Sync** (history replay) and **Initial Size Handshake** for new clients.
- **`src/session.rs`**: Session lifecycle management. Includes `SessionRegistry` and `monitor_session`.
- **`src/pty_manager.rs`**: Direct OS interface for PTY creation and control. Includes **POSIX Process Group** management for robust cleanup of background tasks and a **Watcher Thread** safety net to handle parent process abrupt termination using `nix` and `libc`. Implements `Drop` for automatic resource deallocation.

## Frontend (HTML/JS)
- `frontend/dist/index.html`: Giao diện người dùng duy nhất (SPA). Sử dụng Tailwind CSS cho layout và Xterm.js cho terminal.
- **Tính năng nổi bật:**
  - **3-Tier Rendering Engine**: Hệ thống tự động chọn renderer tối ưu nhất theo thứ tự: **WebGL** (60fps performance) -> **Canvas** (Stable high-perf) -> **DOM** (Fallback).
  - **Adaptive Font Size:** Tự động điều chỉnh cỡ chữ terminal (15px cho desktop, 14px cho mobile).
  - **Robust Layout Engine:** Sử dụng `@xterm/addon-fit` chính chủ với cơ chế retry thông minh để đảm bảo terminal luôn full width/height ngay cả khi renderer khởi động chậm.
  - **Dynamic Viewport Scaling:** Sử dụng `VisualViewport API` để thay đổi chiều cao ứng dụng khi bàn phím mobile hiện lên, ngăn chặn việc che khuất nội dung.
  - **Sticky Modifiers:** Hỗ trợ phím chức năng ảo (Ctrl, Alt) trên mobile.
  - **Real-time Sync:** Tự động cập nhật danh sách session qua SSE.
  - **Single Binary**: Tích hợp sâu với backend qua `rust-embed`.
  - **Robust IME Handling**: Ngăn chặn việc lặp ký tự khi gõ Telex trên mobile (Gboard/Android) bằng cách block data transmission trong quá trình `composition`. Tự động gắn listeners vào hidden textarea với cơ chế retry thông minh.
  - **Smart Clipboard:** Hỗ trợ Ctrl+C (Copy khi có selection, SIGINT khi không có) và Ctrl+V (Paste) thông minh, giúp trải nghiệm web terminal như ứng dụng native.
  - **Mobile Touch Selection**: Hỗ trợ cử chỉ "Long Press" để chọn văn bản trên mobile (bridge to mouse events).

## Testing & Quality
- **Coverage**: **95.12% line coverage** achieved for the Rust backend.
- `tests/sse_integration.rs`: New integration tests for SSE event streaming.
- `tests/password_change_integration.rs`: **New** Verification of mandatory password change flow and forbidden access enforcement.
- `e2e/tests/auth.spec.ts`: **New** Verification of login redirection, credentials validation, session persistence, and forced password change.
- `e2e/tests/clipboard.spec.ts`: **New** Verification of Smart Ctrl+C/V logic and clipboard permissions.
- `e2e/tests/sync-management.spec.ts`: Verification of multi-device synchronization.
- `e2e/tests/shared-resize.spec.ts`: **New** Verification of MIN-Dimension logic (Smallest Screen Priority) across multiple browsers.
- `e2e/tests/renderer-optimization.spec.ts`: **New** Verification of flicker prevention, integer pixel snapping, and high-frequency streaming stability.
- `e2e/tests/vietnamese-ime.spec.ts`: **New** Verification of Vietnamese Telex typing without duplication.
- `e2e/tests/terminal-layout.spec.ts`: Verification of full-screen layout and resizing.
- `e2e/tests/terminal-advanced.spec.ts`: Advanced verification for PTY resizing, special keys (Ctrl+C), session persistence, and UTF-8 rendering.
- `e2e/tests/mobile-ui.spec.ts`: **New** Verification of mobile touch selection, viewport scaling, and keyboard handling.