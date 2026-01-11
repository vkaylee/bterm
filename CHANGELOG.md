# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- **Terminal Layout Fix**: Resolved a critical layout issue where the terminal would not fill the container width. Upgraded to the official `@xterm/addon-fit` (v0.8.0) to ensure compatibility with `xterm` v5.x and fixed internal property access errors (`_viewport`).
- **Robust Fit Logic**: Implemented a retry mechanism in the frontend to ensure the terminal resizing logic (`fit()`) only executes when the renderer viewport is fully initialized, preventing crashes and race conditions.
- **Terminal Layout Tests**: Added `e2e/tests/terminal-layout.spec.ts` to verify full-width/height utilization (98%+) and window resize responsiveness across all rendering tiers.
- **Desktop Font Adjustment**: Optimized the desktop terminal font size to **15px** (previously 16px) for better information density while maintaining readability.

### Added
- **3-Tier Renderer Fallback**: Implemented a robust rendering strategy that prioritizes WebGL (via `@xterm/addon-webgl`) for maximum performance, automatically falls back to Canvas (`@xterm/addon-canvas`) if WebGL fails, and finally to the DOM renderer for universal compatibility.
- **Robust Feature Detection**: Added proactive context checks (`webgl2`, `2d`) to prevent silent failures and race conditions during renderer initialization.
- **Renderer E2E Suite**: Added `e2e/tests/renderer-fallback.spec.ts` to verify automatic failover logic across all three rendering tiers.

### Changed
- **Dependency Upgrade**: Migrated from the deprecated `xterm` package to the modern `@xterm/xterm` (v5.5.0) and its official addons.
- **Visual Viewport Scaling**: The mobile UI now dynamically adjusts the application height based on the `visualViewport` height. This ensures the terminal remains fully visible when the software keyboard appears and prevents overlapping with UI elements.
- **Auto-Scroll to Cursor**: The terminal now automatically scrolls to the bottom when the visual viewport is resized, keeping the command prompt in view.
- **Enhanced Mobile E2E Suite**: Added test cases to verify application scaling, visual viewport response, and performance-optimized listeners.
- **Mobile Performance Optimization**: Implemented CSS hardware acceleration and `requestAnimationFrame` throttling for viewport events to eliminate touch and scroll lag on mobile devices.

### Changed
- **Improved E2E Isolation**: Fixed a data leakage issue in `session-management.spec.ts` where sessions were not properly cleaned up between tests, ensuring more reliable parallel execution.
- **Robust Mobile Layout**: Replaced CSS transforms with dynamic container height adjustments for better cross-browser compatibility on mobile.

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

### [Unreleased]
- **Real-time Synchronization**: Implemented Server-Sent Events (SSE) at `/api/events` to propagate session lifecycle events (created/deleted) to all active dashboards.
- **Automatic Exit Propagation**: Refactored `monitor_session` to broadcast a `SessionDeleted` event when a shell process exits, ensuring all devices are instantly updated.
- **Improved State Management**: `SessionRegistry` now holds a global broadcast sender for consistent event emission across the application.
- **Sync Integration Tests**: Added `tests/sse_integration.rs` and expanded E2E suite to cover multi-device state consistency.
- **Code Health**: Resolved all Clippy warnings and compiler noise related to custom CFG flags.

### Changed
- Updated `src/main.rs` to use `tokio::net::TcpListener` with a fallback loop instead of a hardcoded address.
- Refactored E2E tests to use a custom `test` fixture with dynamic `baseURL` instead of a global `webServer`.
- Improved `tests/port_integration.rs` to be robust against external port occupation by using dynamic collision detection.
- Refined E2E test suite with explicit async synchronization and `networkidle` waits to prevent race conditions.
- **Race Condition Mitigation**: Implemented default-hidden UI sections via inline styles to prevent "flicker" and timing issues during page initialization.

## [0.1.0] - 2026-01-11


