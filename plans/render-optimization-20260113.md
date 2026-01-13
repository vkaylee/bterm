# Implementation Plan: Stutter-Free Terminal Rendering

### Approach
- **Throttling with `requestAnimationFrame` (RAF):** Wrap high-frequency UI updates (`fit`, `scrollToBottom`) into RAF blocks to sync with screen refresh rates.
- **CSS Integer Snapping:** Force terminal container dimensions to whole pixels to prevent sub-pixel rendering flicker.

### Steps
1. **Modify frontend/dist/index.html**:
   - Implement throttled `fit()` and `scrollToBottom()` using RAF.
   - Update `SetSize` logic to use integer pixel values.
   - Refactor `doFit` to be more resilient and less frequent.
2. **Verification**:
   - Manual check with high-speed output.
   - Run `e2e/tests/shared-resize.spec.ts`.

### Timeline
- Implementation: 30 min
- Verification: 20 min
- Total: 50 min
