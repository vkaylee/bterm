# Codebase Summary

## Backend (Rust)
- `src/main.rs`: Entry point, Axum server setup, and static asset embedding.
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

## Testing
  - **`session-management.spec.ts`**: CRUD operations on sessions.
  - **`terminal-interaction.spec.ts`**: Data flow, shell execution, and **auto-exit verification**.
  - **`mobile-ui.spec.ts`**: Responsive design, visual viewport handling, and **input accessibility**.

