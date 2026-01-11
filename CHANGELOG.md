# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- **Dynamic Port Selection**: The server now checks the `PORT` environment variable, defaults to `3000`, and automatically falls back to an available random port if the preferred ports are occupied.
- **Port Integration Tests**: New test suite in `tests/port_integration.rs` to verify binding logic and environment variable overrides.
- **Dynamic E2E Isolation**: Playwright now uses worker-scoped fixtures to spawn independent backend instances on random ports, ensuring zero state leakage during parallel testing.
- **Enter Key Support**: Users can now create sessions by pressing "Enter" in the session name input field.
- **Adaptive Dashboard UI**: The "Active Sessions" section is now automatically hidden when no sessions are active, providing a cleaner initial workspace.
- **Auto-Focus**: The session name input field is automatically focused on page load if no active sessions exist.
- **Adaptive Virtual Keyboard**: On mobile, the custom virtual keyboard (Esc, Ctrl, Alt, etc.) now automatically hides when the system software keyboard is dismissed, maximizing screen space for the terminal.

### Added
- **Lib/Bin Architecture**: Refactored the backend into a library (`src/lib.rs`) and a thin binary (`src/main.rs`) to enable high-fidelity integration testing of all modules.
- **WebSocket Integration Tests**: New test suite in `tests/ws_integration.rs` and unit tests in `src/ws.rs` to verify full terminal data flow, history transmission, and error handling.
- **Static Asset Testing**: Added automated verification for embedded asset serving and MIME type detection.
- **Session History Test**: Implemented verification for automatic state recovery on WebSocket reconnection.
- **Enhanced Coverage**: Achieved 94.12% line coverage across the Rust backend, ensuring all critical business logic and error paths are verified.

### Changed
- Updated `src/main.rs` to use `tokio::net::TcpListener` with a fallback loop instead of a hardcoded address.
- Refactored E2E tests to use a custom `test` fixture with dynamic `baseURL` instead of a global `webServer`.
- Improved `tests/port_integration.rs` to be robust against external port occupation by using dynamic collision detection.
- Refined E2E test suite with explicit async synchronization and `networkidle` waits to prevent race conditions.
- **Race Condition Mitigation**: Implemented default-hidden UI sections via inline styles to prevent "flicker" and timing issues during page initialization.

## [0.1.0] - 2026-01-11


