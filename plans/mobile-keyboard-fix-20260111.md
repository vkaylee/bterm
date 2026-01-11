# Implementation Plan: Mobile Terminal Keyboard UX Fix

### ## Approach
- **Why this solution:** Instead of using CSS `transform` to move the control bar (which doesn't affect the layout flow), we will dynamically adjust the height of the main `#app` container to match the `window.visualViewport.height`. This forces the browser to recalculate the flex layout, shrinking the terminal content and naturally pushing the control bar above the keyboard. Since the terminal's actual height changes, `xterm-addon-fit` will correctly resize the terminal buffer, and `xterm.js` will naturally scroll the cursor into view.
- **Alternatives considered:** 
    - `Manual scrolling`: Only scrolls the text but doesn't prevent the control bar from overlapping.
    - `Fixed positioning`: Harder to manage with flexbox and might cause issues with different browser toolbars.

### ## Steps

1. **Update `updateVisualViewport` in `frontend/dist/index.html`** (15 min)
   - Modify the logic to set `document.getElementById('app').style.height`.
   - Remove `transform: translateY` on `#control-bar`.
   - Ensure `#control-bar` visibility is handled correctly.
   - Explicitly call `fitAddon.fit()` and `term.scrollToBottom()`.

2. **Refine CSS for `#app`** (5 min)
   - Ensure the container handles height transitions smoothly if needed (though immediate is usually better for keyboard).

3. **Verification** (10 min)
   - Since I cannot easily simulate a mobile keyboard in this environment, I will rely on logic verification and ensure no regressions in desktop view.

### ## Timeline
| Phase | Duration |
|-------|----------|
| Implementation | 15 min |
| Refinement | 5 min |
| Verification | 10 min |
| **Total** | **30 min** |

### ## Rollback Plan
- Revert changes in `frontend/dist/index.html` to the previous version using `git checkout`.

### ## Security Checklist
- [x] Input validation (N/A for this UI change)
- [x] Auth checks (N/A)
- [x] Error handling: Added try/catch around `fit()` calls.
