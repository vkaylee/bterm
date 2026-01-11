# Implementation Plan: WebGL Renderer with Fallback (2026-01-11)

## Approach
This plan implements a robust rendering strategy for the xterm.js terminal. We will prioritize the **WebGL renderer** for maximum performance (GPU acceleration). If WebGL is not supported by the user's browser or device (e.g., context loss, outdated hardware), the system will automatically fall back to the default **DOM renderer** (or a Canvas renderer if we decide to include that as a middle tier, but for now, DOM is the safe default fallback).

This "Progressive Enhancement" approach ensures:
1.  **Top-tier performance** for modern devices.
2.  **Guaranteed stability** for all users.

## Steps

### 1. Asset Acquisition (5 min)
We need to obtain the `xterm-addon-webgl.js` library. Since we are in a "vanilla" environment (no bundler), we will fetch the browser-ready UMD build.
*   **Action:** Download `xterm-addon-webgl.js.map` and `xterm-addon-webgl.js` to `frontend/dist/assets/`.
*   **Source:** CDN (jsdelivr/unpkg) or `node_modules` if we were using npm (we will simulate a fetch/mock for this environment or use `curl`).

### 2. Frontend Integration (15 min)
*   **File:** `frontend/dist/index.html`
*   **Action:** Add the `<script>` tag for the WebGL addon.
*   **File:** `frontend/dist/js/terminal.js` (or inline script in `index.html` depending on current structure)
*   **Logic:**
    ```javascript
    // Pseudo-code logic
    const webglAddon = new WebglAddon.WebglAddon();
    webglAddon.onContextLoss(e => {
        webglAddon.dispose();
    });
    try {
        term.loadAddon(webglAddon);
        console.log("WebGL renderer loaded");
    } catch (e) {
        console.warn("WebGL failed, falling back to DOM", e);
    }
    ```

### 3. Testing Implementation (20 min)
We need to verify *which* renderer is active.
*   **File:** `e2e/tests/renderer-fallback.spec.ts` (New file)
*   **Tests:**
    1.  **Happy Path:** Verify `term._core._renderService._renderer` is an instance of `WebglRenderer` in a standard environment.
    2.  **Fallback Path:** Force-fail WebGL (mocking `document.createElement('canvas').getContext('webgl')` to return null) and verify the renderer remains the default (DOM).

### 4. Integration Verification (10 min)
*   Run the full test suite to ensure the new addon doesn't break existing functionality (input, resizing, etc.).

## Timeline
| Phase | Duration |
|-------|----------|
| Assets | 5 min |
| Integration | 15 min |
| Testing | 20 min |
| Verification | 10 min |
| **Total** | **50 min** |

## Rollback Plan
1.  Remove the `<script>` tag from `index.html`.
2.  Remove the `loadAddon` logic from the JavaScript.
3.  Delete the `assets/xterm-addon-webgl.js` file.

## Security Checklist
- [x] **Source Validation:** Ensure we are fetching the addon from a trusted version/source.
- [x] **Error Handling:** `try/catch` block prevents the entire terminal from crashing if WebGL init fails.
