# Codebase Summary

## Backend (Rust)
- `src/lib.rs`: The core library containing application logic, routing, and shared modules.
- `src/main.rs`: Thin binary wrapper. Sets up the Axum server with dynamic port selection (Env > 3000 > Random) and serves the app.
- `src/api.rs`: REST API endpoints for session management (List, Create, Delete).
- `src/ws.rs`: WebSocket handler for terminal I/O, PTY resizing, and session history synchronization.
- **`src/session.rs`**: Session lifecycle management. Includes `SessionRegistry` for active session tracking and `monitor_session` for resource cleanup.
- **`src/pty_manager.rs`**: Direct OS interface for PTY creation and control.

## Frontend (Single-page Application)
... (UI details omitted for brevity) ...

## Testing & Quality
- **Coverage**: **94.12% line coverage** achieved for the Rust backend via `cargo-tarpaulin`.
- `tests/ws_integration.rs`: Full WebSocket/PTY data flow integration tests.
- `tests/port_integration.rs`: Integration tests for dynamic port binding with collision detection.
- `e2e/playwright.config.ts`: Configuration for parallelized E2E tests using worker-scoped isolation.
- `e2e/fixtures.ts`: Worker fixtures managing isolated backend instances for E2E tests.
- `e2e/tests/`: Playwright suite covering session management, terminal interactions, and mobile adaptive UI.

