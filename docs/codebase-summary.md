# Codebase Summary

## Backend (Rust)
- `src/main.rs`: Entry point, Axum server setup, and static asset embedding.
- `src/api.rs`: REST API endpoints for session management (List, Create, Delete).
- `src/ws.rs`: WebSocket handler for terminal I/O and PTY resizing.
- `src/pty_manager.rs`: Manages the lifecycle of PTY instances.
- `src/session.rs`: Logic for shared session state and broadcasting.

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

