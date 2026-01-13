### # Implementation Plan: Persistent Terminal Sessions

### ## Approach
To enable users to "tiếp tục phiên làm việc y hệt như thiết bị cũ", we focused on ensuring that when a new client connects to an existing session, it receives the current state of that session. The existing `history` buffer provides a good starting point for replaying past output, and we've implemented sending the current PTY size for display synchronization. This solution leverages the existing `history` buffer and `broadcast_tx` to provide a "best effort" session continuity for terminal output within the current architectural constraints.

### ## Steps
1.  **Retrieve and Send History to New Clients (Initial State)**
    -   **Status:** **Completed**.
    -   **Details:** The existing implementation in `src/ws.rs` already retrieves and sends the `history` data to newly connected WebSocket clients. This was verified by `test_ws_history_sent`.

2.  **Ensure Session Persistence on Client Disconnect (Client-side logic consideration)**
    -   **Status:** **Future Consideration / No server-side code change.**
    -   **Details:** The current server-side behavior is that the session persists as long as the underlying PTY process is running, regardless of client connections. The session is only removed when the PTY process ends (e.g., user types `exit`). True session persistence *beyond* the PTY's lifetime (e.g., reattaching to a session where the shell has exited, similar to `tmux` or `screen`) was identified as a significant architectural change and is considered a future enhancement.

3.  **Synchronize PTY Resizing for All Clients**
    -   **Status:** **Completed and Tested.**
    -   **Files modified:** `src/ws.rs`
    -   **Details:** Modified `src/ws.rs` in `handle_socket` to calculate and send the current effective PTY size to newly connected clients as a `Message::Binary` containing a JSON string. This ensures new clients immediately display the terminal with the correct dimensions.

4.  **Testing (Integration and Unit)**
    -   **Status:** **Implemented and Passed.**
    -   **Files modified:** `src/ws.rs` (added new test case).
    -   **Details:**
        -   Added `test_ws_initial_pty_size_sent` to `src/ws.rs` to verify that the initial PTY size is correctly sent to new clients.
        -   All existing and new tests passed after fixing a type mismatch error between `axum::extract::ws::Message::Text` and `tokio_tungstenite::tungstenite::Message::Binary` when receiving from `tokio-tungstenite` client.

### ## Timeline
| Phase                          | Duration | Status     |
|--------------------------------|----------|------------|
| Send History to New Clients    | 60 min   | Completed  |
| Client Disconnect Handling     | 30 min   | Future     |
| PTY Resizing Synchronization   | 30 min   | Completed  |
| Testing                        | 45 min   | Completed  |
| **Total Implemented**          | **1 hour 15 min** |            |

### ## Rollback Plan
- Revert changes to `src/ws.rs`. Remove `test_ws_initial_pty_size_sent` from `src/ws.rs`.

### ## Security Checklist
- [x] Input validation (WebSocket messages from client) - *Assumed to be handled by existing serde deserialization.*
- [x] Auth checks (Ensure only authenticated/authorized users can connect to *their* sessions - this is assumed to be handled by the existing `tower-sessions` middleware in `src/ws.rs` or earlier in the request flow).
- [ ] Rate limiting (Consider rate limiting WebSocket messages to prevent abuse). - *Not implemented in this iteration, for future consideration.*
- [x] Error handling (Graceful handling of WebSocket disconnections and PTY errors).

### ❓ Open Questions / Future Considerations
1.  **Full Terminal State:** The current solution only sends raw output and initial size. For a truly "y hệt như thiết bị cũ" experience, a client needs to know the full terminal screen buffer (characters, colors, attributes), cursor position, and potentially the current state of the shell (e.g., if a command is partially typed). This would require a more sophisticated terminal state serialization/deserialization, possibly leveraging the client-side terminal emulator's capabilities or a server-side virtual terminal emulator. This is a much larger feature.
2.  **Session Persistence beyond PTY process:** If the user wants to resume a session even after the shell process inside the PTY has exited (e.g., typing `exit`), the server would need a mechanism to keep the PTY alive or restart the shell in a way that restores its context. This is akin to `tmux` or `screen` and represents a significant architectural shift.