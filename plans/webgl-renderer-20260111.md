# Plan: WebGL Renderer Addon Implementation (2026-01-11)

## Goal
Tăng tốc độ render terminal lên mức tối đa bằng cách chuyển từ DOM sang GPU (WebGL), giải quyết triệt để hiện tượng lag trên mobile.

## Implementation Steps

### 1. Download Asset
Tải file `xterm-addon-webgl.js` phiên bản tương thích (UMD) và lưu vào `frontend/dist/assets/`.

### 2. Update index.html
Nhúng addon:
```html
<script src="./assets/xterm-addon-webgl.js"></script>
```

Khởi tạo trong hàm `joinSession`:
```javascript
const webglAddon = new WebglAddon.WebglAddon();
try {
    term.loadAddon(webglAddon);
} catch (e) {
    console.warn("WebGL addon failed to load, falling back to DOM renderer", e);
}
```

## Success Criteria
- [ ] `window.term._core._renderService._renderer.constructor.name` trả về `WebglRenderer`.
- [ ] Thao tác scroll và hiển thị dữ liệu lớn mượt mà hơn rõ rệt trên mobile emulator.
- [ ] Tất cả E2E tests cũ (terminal interaction) đều pass.
