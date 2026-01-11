# Codebase Summary

## Backend (Rust)
- `src/main.rs`: Entry point. Sets up the Axum server with dynamic port selection (Env > 3000 > Random), configures CORS, and serves embedded static assets.
- `src/api.rs`: REST API endpoints for session management (List, Create, Delete).
- `src/ws.rs`: WebSocket handler for terminal I/O and PTY resizing.
- **`src/session.rs`**: Quản lý vòng đời session. Bao gồm `SessionRegistry` để lưu trữ các session đang hoạt động và hàm `monitor_session` để quản lý lịch sử và tự động dọn dẹp tài nguyên khi shell kết thúc.
- **`src/pty_manager.rs`**: Giao tiếp trực tiếp với hệ điều hành để tạo và điều khiển PTY (Pseudo-Terminal).

## Frontend (Single-page Application)
- `frontend/dist/index.html`: Main UI built with Tailwind CSS and Xterm.js.
  - **Header**: Active session info, connection pulse, and system status.
  - **Dashboard**: Content-first list of active sessions with a "Start New Instance" section. The active sessions list is automatically hidden when empty to maintain focus.
  - **Auto-Join**: Successful session creation immediately triggers the terminal view for seamless UX.
  - **Keyboard & Focus UX**: Supports "Enter" key for rapid session creation and automatically focuses the input field on empty dashboards.
  - **Terminal View**: Xterm.js container with a mobile-optimized 2-row virtual keyboard.
  - **Mobile Logic**: Uses `VisualViewport API` for layout adjustment and "Sticky Modifiers" for complex key combinations.

## Testing & Quality
- `tests/port_integration.rs`: Robust integration tests for dynamic port binding with automatic collision detection.
- `e2e/playwright.config.ts`: Configuration for parallelized E2E tests using worker-scoped isolation.
- `e2e/fixtures.ts`: Worker fixtures that manage the lifecycle of isolated backend instances for tests.
- `e2e/tests/`: Suite covering session management (including Enter key flow), terminal data flow, and mobile UI responsiveness.

