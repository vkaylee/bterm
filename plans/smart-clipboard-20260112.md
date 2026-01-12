# Implementation Plan: Smart Clipboard (Ctrl+C/V)

## Approach
Implement "Smart Copy/Paste" logic directly in the xterm.js initialization.
- **Ctrl+C**: If text is selected, allow browser default (Copy). If no selection, send `Ctrl+C` signal to backend.
- **Ctrl+V**: Allow browser default (Paste), which xterm.js handles by reading the hidden textarea.

## Steps
1.  **Modify Frontend** (15 min)
    -   File: `frontend/dist/index.html`
    -   Action: Add `term.attachCustomKeyEventHandler` logic inside `joinSession`.

2.  **Testing** (20 min)
    -   File: `e2e/tests/clipboard.spec.ts`
    -   Action: Create E2E test to verify:
        -   Selection + Ctrl+C = No SIGINT (Copy).
        -   No Selection + Ctrl+C = SIGINT.
        -   Ctrl+V = Paste.

## Timeline
| Phase | Duration |
|-------|----------|
| Implementation | 15 min |
| Testing | 20 min |
| **Total** | **35 min** |

## Security Checklist
- [x] No sensitive data logged.
- [x] Clipboard access respects browser permissions (relying on native browser behavior for Ctrl+C/V reduces permission issues vs `navigator.clipboard` API calls).
