# Codebase Summary

## Backend (Rust)
- `src/lib.rs`: The core library containing application logic, routing, and shared modules.
- `src/main.rs`: Thin binary wrapper. Sets up the Axum server with dynamic port selection (Env > 3000 > Random) and serves the app.
- `src/api.rs`: REST API endpoints for session management (List, Create, Delete).
- `src/ws.rs`: WebSocket handler for terminal I/O, PTY resizing, and session history synchronization.
- **`src/session.rs`**: Session lifecycle management. Includes `SessionRegistry` (now integrated with global broadcast) and `monitor_session` for resource cleanup and real-time deletion notifications.
- **`src/pty_manager.rs`**: Direct OS interface for PTY creation and control.

## Frontend (Single-page Application)
- `frontend/dist/index.html`: Main UI built with Tailwind CSS and Xterm.js.
  ... (UI details) ...
  - **Real-time Sync**: Uses `EventSource` to listen for backend events and automatically update the dashboard or redirect users on session termination.

## Testing & Quality
- **Coverage**: **95.12% line coverage** achieved for the Rust backend.
- `tests/sse_integration.rs`: New integration tests for SSE event streaming.
- `e2e/tests/sync-management.spec.ts`: Verification of multi-device synchronization.

