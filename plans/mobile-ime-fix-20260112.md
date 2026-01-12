# Plan: Fix Mobile IME Composition (Vietnamese Telex/Gboard)

## Problem
On mobile devices (specifically Android with Gboard using Vietnamese Telex), typing complex characters (like "â", "đ", "ư") fails or produces double characters. This is because mobile keyboards use IME composition heavily (sending intermediate characters like 'a', 'a' before 'â'), and the current Xterm.js integration immediately sends these intermediate keystrokes to the server via `term.onData`.

## Root Cause
- Mobile browsers fire `compositionstart`, `compositionupdate`, and `compositionend` events.
- During composition, Xterm.js may emit `onData` events for intermediate states (or `keyCode 229`).
- The backend receives these partial inputs ('a', 'a') instead of waiting for the finalized character ('â').

## Solution: "Block & Wait" Approach
We will intercept the composition lifecycle to suppress data transmission during the composition phase, allowing only the final committed text to be sent.

### Technical Implementation

1.  **State Management:**
    -   Add a global flag `let isComposing = false;` inside the script.

2.  **Event Listeners (in `frontend/dist/index.html`):**
    -   Attach to the Xterm helper textarea (`.xterm-helper-textarea`).
    -   **`compositionstart`**: Set `isComposing = true`.
    -   **`compositionend`**:
        -   Set `isComposing = false`.
        -   **Crucial:** Mobile browsers often fire `compositionend` *before* the final `input` event that triggers `term.onData`. By setting the flag to false here, we allow the subsequent `onData` (containing the correct final char) to pass through.
        -   *Fallback:* If Xterm doesn't fire `onData` automatically after composition (rare but possible on some WebViews), we might need to manually send `event.data`. For now, we assume Xterm 5.x handles the `input` event correctly after composition ends.

3.  **Modify `term.onData`:**
    -   Add a check: `if (isComposing) return;`
    -   This effectively filters out all intermediate noise from Gboard.

### Steps

1.  **Modify `frontend/dist/index.html`**:
    -   Locate the IME optimization section.
    -   Define `isComposing`.
    -   Add `compositionstart` and `compositionend` listeners.
    -   Update `term.onData` logic.

### Verification
-   **Manual Test (Mobile):** Open the app on Android (or simulator), switch to Vietnamese Telex, type "aa" -> expecting "â" on screen, not "aa" or "aâ".
-   **Manual Test (PC):** Ensure Unikey still works (it usually doesn't trigger browser composition events in the same way, or handles them fast enough).
-   **Regression:** Check normal typing (English) isn't delayed.

## Rollback
-   Revert changes to `frontend/dist/index.html`.
