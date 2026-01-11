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
  - **Dashboard**: Content-first list of active sessions with a "Start New Instance" section at the bottom.
  - **Auto-Join**: Successful session creation immediately triggers the terminal view for seamless UX.
  - **Terminal View**: Xterm.js container with a mobile-optimized 2-row virtual keyboard.
  - **Mobile Logic**: Uses `VisualViewport API` for layout adjustment and "Sticky Modifiers" for complex key combinations.

## Testing & Quality
- `tests/port_integration.rs`: Integration tests for dynamic port binding and env overrides.
- `e2e/playwright.config.ts`: Configuration for Playwright E2E tests with parallel worker support.
- `e2e/fixtures.ts`: Worker-scoped fixtures for spawning isolated backend instances during E2E tests.
- `e2e/tests/`: Comprehensive E2E test suite covering Session Management, Terminal Interaction, Assets, and Mobile UI.

