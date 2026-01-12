# Implementation Plan: Local Authentication with Session Cookies

## Approach
We will implement a self-contained authentication system using **SQLite** for user persistence and **Session Cookies** for state management. This ensures security (HttpOnly cookies) and simplicity (self-hosted friendly).

### Tech Stack
- **Database:** `sqlx` with `sqlite` (Embedded, zero-config).
- **Hashing:** `argon2` (Industry standard).
- **Sessions:** `tower-sessions` (Middleware for cookie management).
- **Backend:** `axum` (Existing).

## Steps

### 1. Dependencies (5 min)
Add crates to `Cargo.toml`:
- `sqlx` (sqlite, runtime-tokio)
- `argon2`
- `tower-sessions`
- `tokio` (ensure features)

### 2. Database Layer (`src/db.rs`) (15 min)
- Initialize SQLite connection pool.
- Create `users` table if not exists (Auto-migration on startup).
  ```sql
  CREATE TABLE IF NOT EXISTS users (
      id INTEGER PRIMARY KEY,
      username TEXT NOT NULL UNIQUE,
      password_hash TEXT NOT NULL,
      role TEXT DEFAULT 'member'
  );
  ```
- Implement `create_user` and `get_user_by_username`.

### 3. Auth Logic (`src/auth.rs`) (20 min)
- **Hashing:** Helper functions to hash/verify passwords.
- **Handlers:**
  - `login`: Verify credentials -> `session.insert("user_id", id)`.
  - `logout`: `session.flush()`.
  - `me`: Return current user info.
- **Middleware:** `require_auth` layer for protected routes.

### 4. Integration (`src/lib.rs` & `src/main.rs`) (15 min)
- Initialize `SqlitePool` in `main.rs`.
- Add `SessionManagerLayer` to the router.
- Mount auth routes (`/api/auth/*`).
- Protect `/api/sessions` and `/ws` with auth middleware.

### 5. Frontend Update (Placeholder) (10 min)
- For the MVP, we will serve a simple static `login.html` if the user is not authenticated, or handle 401 in the current SPA.
- *Note:* This plan focuses on Backend. Frontend integration will be minimal (basic login form).

## Timeline
| Phase | Duration |
|-------|----------|
| Dependencies | 5 min |
| DB Layer | 15 min |
| Auth Logic | 20 min |
| Integration | 15 min |
| Testing | 10 min |
| **Total** | **1 hour 5 min** |

## Security Checklist
- [ ] Passwords hashed with Argon2id.
- [ ] Cookies: `HttpOnly`, `SameSite=Strict`, `Secure` (in prod).
- [ ] Rate limiting (Optional for MVP, recommended later).
