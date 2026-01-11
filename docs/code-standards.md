# Code Standards

Dự án BTerminal tuân thủ các tiêu chuẩn mã nguồn nghiêm ngặt để đảm bảo tính ổn định và bảo mật.

## Rust Backend

### 1. Linting & Safety
- **Strict Mode:** Tất cả các tệp tin phải bắt đầu bằng `#![deny(warnings)]`. Cảnh báo được coi là lỗi biên dịch.
- **Clippy:** Sử dụng các nhóm rules `clippy::all`, `clippy::pedantic`, và `clippy::nursery`.
- **Error Handling:** Luôn sử dụng `anyhow` cho các hàm có thể gây lỗi để quản lý context dễ dàng hơn.

### 2. Concurrency
- Tránh giữ `MutexGuard` qua các điểm `await` (gây lỗi không thực hiện được `Send`). Luôn giải phóng lock trước khi gọi các hàm async.

## Frontend

### 1. Assets Management
- **No CDN:** Không sử dụng CDN bên ngoài. Toàn bộ JS/CSS/Fonts phải được lưu tại `frontend/dist/assets/` và nhúng vào binary qua `rust-embed`.
- **Styling:** Sử dụng Tailwind CSS (Standalone build).

### 2. Terminal Rendering
- **Font:** Sử dụng JetBrains Mono. Cỡ chữ mặc định là **16px** cho Desktop (viewport >= 640px) và **14px** cho Mobile để tối ưu hóa không gian hiển thị.
- **Responsive:** Sử dụng `dvh` thay cho `vh` và kết hợp với `VisualViewport API` để điều chỉnh `app.style.height`. Điều này đảm bảo terminal không bị bàn phím ảo che khuất.
- **Resize:** Luôn sử dụng `requestAnimationFrame` trong `ResizeObserver` và gọi `term.scrollToBottom()` sau khi resize để đảm bảo con trỏ luôn nằm trong vùng nhìn thấy.
