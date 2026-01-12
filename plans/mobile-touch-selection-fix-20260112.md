### # Implementation Plan: Mobile Terminal Touch Selection Fix

### ## Mode: HARD (Complex UI/Interaction Fix)

### Root Cause Analysis
1. **Symptom:** Users cannot select text in the terminal on mobile devices by long-pressing.
2. **Reproduction:** Open BTerminal on a mobile browser, try to long-press on any text in the terminal. The browser either does nothing or tries to scroll/zoom, but no text selection handles appear.
3. **Affected Components:** `frontend/dist/index.html` (terminal initialization and event handling), `xterm.js` (selection logic).
4. **Root Cause:** `xterm.js` uses Canvas/WebGL for rendering. These renderers do not create DOM text nodes, so the browser's native text selection (magnifier/handles) has no "text" to target. `xterm.js` has its own selection logic but it is primarily mapped to Mouse Events (`mousedown`, `mousemove`), which aren't automatically triggered by long-press gestures on mobile.

### Investigation Steps
1. **Verify Event Interception:** Check if `touch-action: manipulation` is blocking or allowing necessary events.
2. **Test DOM Renderer Fallback:** Temporarily force DOM renderer to see if native selection works (it usually doesn't without extra config, as xterm still intercepts).
3. **Analyze Scroll Conflicts:** Ensure that enabling selection doesn't break the ability to scroll the terminal buffer.

### Fix Plan (Approach: Event Emulation Bridge)
We will implement a touch-to-mouse bridge specifically for "long press" detection.

**Files to modify:** `frontend/dist/index.html`

**Code changes:**
- Add `touchstart`, `touchmove`, and `touchend` listeners to the terminal container.
- Use a `longPressTimer` to detect a hold (>500ms).
- If a long press is detected:
    - Call `ev.preventDefault()` to stop the browser from scrolling.
    - Dispatch a synthetic `mousedown` event to the xterm screen.
    - Track subsequent `touchmove` events to dispatch `mousemove` (updating the selection).
    - On `touchend`, dispatch `mouseup`.

### Testing Plan
1. **Manual Verification:** Use Chrome DevTools mobile emulation to verify long-press selection.
2. **E2E Test:** Update/Add a test in `e2e/tests/mobile-ui.spec.ts` to simulate a long press and check for selection.

### ## Timeline
| Phase | Duration |
|-------|----------|
| Investigation | 10 min |
| Implementation | 20 min |
| Testing (Manual + E2E) | 20 min |
| **Total** | **50 min** |

### ## Rollback Plan
- Revert changes in `frontend/dist/index.html`.
- If selection logic causes "stuck" scrolling, ensure `isSelecting` is properly reset on `touchend` and `touchcancel`.
