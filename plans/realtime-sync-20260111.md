# Implementation Plan: Real-time Session Deletion Sync (2026-01-11)

## Approach
Implement a broadcast system using `tokio::sync::broadcast` and expose it via a Server-Sent Events (SSE) endpoint. This ensures all dashboard instances are notified immediately when a session is created or deleted.

## Steps

### 1. Backend Core (src/lib.rs)
- Define `GlobalEvent` enum with `Serialize` derive.
- Update `AppState` to include `tx: broadcast::Sender<GlobalEvent>`.
- Initialize the channel in `main.rs` (or `lib.rs` setup).

### 2. API Updates (src/api.rs)
- Update `create_session` to emit `GlobalEvent::SessionCreated`.
- Update `delete_session` to emit `GlobalEvent::SessionDeleted`.
- Implement `events_handler` endpoint at `GET /api/events`.

### 3. Frontend Updates (frontend/dist/index.html)
- Add `initEventSource()` function.
- Handle `SessionDeleted`: Remove element from DOM or call `refreshSessions()`.
- Handle `SessionCreated`: Call `refreshSessions()`.

### 4. Verification
- Manual test with two browser windows.
- Add integration test for SSE event reception.

## Rollback
- Remove the `/api/events` route and broadcast sender from `AppState`.
