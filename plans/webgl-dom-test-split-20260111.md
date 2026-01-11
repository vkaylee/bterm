# Implementation Plan: WebGL/DOM Test Split & Reliability Fixes

### ## Approach
- **Project Splitting:** Use Playwright Projects to run the same test suite against two configurations: one with the default WebGL renderer and one forcing the DOM renderer.
- **Renderer Forcing:** Add a query parameter `?renderer=dom` to `index.html` logic to simulate a WebGL failure or manual override.
- **Reliability:** Address race conditions in session management and terminal interaction tests.

### ## Steps

1. **Frontend Support for Manual Renderer Choice** (5 min)
   - Modify `frontend/dist/index.html` to check for `URLSearchParams`.
   - If `renderer=dom` is present, skip WebGL addon loading.

2. **Configure Playwright Projects** (10 min)
   - Modify `e2e/playwright.config.ts`.
   - Define `WebGL` project (default).
   - Define `DOM-Fallback` project (appends `?renderer=dom` to test URLs).

3. **Fix Session Management Flakiness** (15 min)
   - **Problem:** `active-sessions-section` visibility failures due to shared worker state or slow API response.
   - **Fix:** Add a `beforeEach` to `session-management.spec.ts` to ensure all sessions are cleared before tests start, or use unique worker IDs.
   - **Refinement:** Ensure `toBeHidden` properly waits for the initial fetch to settle.

4. **Fix Terminal Interaction Timeouts** (15 min)
   - **Problem:** Output verification timing out.
   - **Fix:** Improve the buffer inspection logic. Ensure WebSocket connection is stable before sending commands.
   - **Mock Improvement:** Ensure `MockWebglRenderer` doesn't block the xterm.js internal state updates.

### ## Timeline
| Phase | Duration |
|-------|----------|
| Frontend Changes | 5 min |
| Config Updates | 10 min |
| Test Reliability Fixes | 30 min |
| Verification | 15 min |
| **Total** | **1 hour** |

### ## Rollback Plan
- Revert `playwright.config.ts` to use a single project.
- Remove query param logic from `index.html`.

### ## Security Checklist
- [x] No secrets exposed.
- [x] Mock code only active in frontend assets, no backend impact.
