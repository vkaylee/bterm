# Plan: Mobile Touch & Scroll Optimization (2026-01-11)

## Root Cause Analysis
- **Main Thread Blockage:** JavaScript xử lý sự kiện viewport và touch không đồng bộ với trình duyệt, gây ra hiện tượng "jank".
- **Lack of Hardware Acceleration:** Terminal DOM có thể chứa nhiều phần tử, việc scroll không được GPU hỗ trợ dẫn đến lag.
- **Non-Passive Listeners:** Browser phải chờ JS kiểm tra `preventDefault()` trước khi scroll.

## Implementation Steps

### 1. CSS Optimization
Thêm style vào `#terminal-container` và các thành phần liên quan:
```css
#terminal-container {
    will-change: transform;
    backface-visibility: hidden;
    touch-action: pan-y; /* Cho phép scroll dọc mặc định mượt hơn */
}
.modifier-key {
    user-select: none;
    -webkit-tap-highlight-color: transparent;
}
```

### 2. JS Event Refactoring
Sửa đổi cách đăng ký sự kiện:
```javascript
// Trước
window.visualViewport.addEventListener('resize', updateVisualViewport);

// Sau (Sử dụng rAF và passive listener)
let tick = false;
function throttledUpdate() {
    if (!tick) {
        requestAnimationFrame(() => {
            updateVisualViewport();
            tick = false;
        });
        tick = true;
    }
}
window.visualViewport.addEventListener('resize', throttledUpdate, { passive: true });
window.visualViewport.addEventListener('scroll', throttledUpdate, { passive: true });
```

## Verification
- Chạy `npx playwright test tests/mobile-ui.spec.ts`.
- Manual check trên thiết bị thật (nếu có).
