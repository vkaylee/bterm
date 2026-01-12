# System Architecture

## Thành phần hệ thống

```ascii
┌──────────────┐      HTTP (JSON)      ┌──────────────────┐
│   Browser    │◄─────────────────────▶│   Axum Server    │
│  (Dashboard) │◄─────────────────────┤ (SessionRegistry)│
└──────────────┘      SSE Events       └────────┬─────────┘
        ▲                                       │
        │ WebSockets (Binary)                   │ Spawn
        ▼                                       ▼
┌──────────────┐                       ┌──────────────────┐
│   Xterm.js   │◄─────────────────────▶│   PTY Instance   │
│  (Terminal)  │    Broadcast Channel  │ (/bin/bash etc.) │
└──────────────┘                       └──────────────────┘
```

## Luồng dữ liệu (Data Flow)
1. **PTY Output -> Clients:**
   ...
2. **Client Input -> PTY:
   ...
3. **Terminal Resizing:**
   ...
4. **Real-time Sync (Dashboard):**
   ...

## Quản lý vòng đời tiến trình (Subprocess Cleanup)

Để ngăn chặn tình trạng rò rỉ tài nguyên (resource leaking) khi các session kết thúc hoặc khi ứng dụng gặp sự cố, BTerminal áp dụng cơ chế quản lý tiến trình 3 lớp dựa trên **POSIX Process Groups (PGID)**:

1. **Dọn dẹp chủ động (Manual Shutdown):** Khi một session bị xóa khỏi registry (ví dụ: qua Dashboard), hệ thống gửi tín hiệu `SIGKILL` tới toàn bộ Process Group của session đó.
2. **Dọn dẹp tự động (Drop Trait):** Khi đối tượng `PtyManager` bị hủy khỏi bộ nhớ (drop), nó tự động gọi hàm `shutdown()` để giải phóng PTY và tiêu diệt các tiến trình liên quan.
3. **Safety Net (Watcher Thread):** Đối với các trường hợp ứng dụng chính bị tắt đột ngột (crash hoặc `kill -9`), BTerminal khởi tạo một background thread cho mỗi session:
    - Thread này sử dụng `libc::getppid()` để theo dõi PID của tiến trình cha.
    - Nếu tiến trình cha biến mất (PID cha đổi thành 1 - init), watcher thread sẽ ngay lập tức thực hiện `Recursive Kill` cho process group con rồi mới kết thúc.

## Smart Clipboard (Shortcuts)

BTerminal triển khai logic xử lý clipboard thông minh để thu hẹp khoảng cách giữa trình duyệt và terminal truyền thống:

-   **Context-Aware Ctrl+C:**
    -   **Chế độ Copy:** Nếu người dùng bôi đen (select) văn bản, phím `Ctrl+C` sẽ kích hoạt lệnh Copy của trình duyệt.
    -   **Chế độ Interrupt:** Nếu không có văn bản nào được chọn, `Ctrl+C` sẽ gửi mã byte `\x03` (SIGINT) tới backend để dừng tiến trình đang chạy.
-   **Native Paste (Ctrl+V):** Cho phép dán văn bản trực tiếp từ clipboard hệ thống vào terminal thông qua phím tắt tiêu chuẩn mà không cần menu chuột phải.

## Frontend Rendering Strategy

BTerminal sử dụng cơ chế render 3 tầng (3-Tier Fallback) để đảm bảo cân bằng giữa hiệu suất và tính tương thích trên mọi thiết bị:

1.  **Tier 1: WebGL Renderer (`@xterm/addon-webgl`)**
    - **Ưu tiên:** Cao nhất.
    - **Cơ chế:** Sử dụng WebGL2 để render ký tự dưới dạng texture trên GPU.
    - **Lợi ích:** Đạt 60fps ổn định ngay cả khi output dồn dập (như lệnh `cat` file lớn). Không gây load cho Main Thread.
2.  **Tier 2: Canvas Renderer (`@xterm/addon-canvas`)**
    - **Ưu tiên:** Thứ hai (Fallback khi không có WebGL).
    - **Cơ chế:** Sử dụng Canvas 2D Context. Nhanh hơn DOM nhưng chậm hơn WebGL.
    - **Lợi ích:** Ổn định cao, hỗ trợ rộng rãi hơn WebGL nhưng vẫn tránh được layout thrashing của DOM.
3.  **Tier 3: DOM Renderer (Core)**
    - **Ưu tiên:** Cuối cùng.
    - **Cơ chế:** Render từng dòng terminal thành các thẻ `<div>` trong DOM.
    - **Lợi ích:** Tương thích tuyệt đối (chạy được trên mọi trình duyệt hỗ trợ JS cơ bản), debugging dễ dàng, hỗ trợ tốt nhất cho Screen Readers.

**Cơ chế chuyển đổi (Failover):**
- Khi khởi tạo, ứng dụng thực hiện **Feature Detection** (kiểm tra `canvas.getContext('webgl2')`).
- Nếu WebGL fail hoặc mất context (context loss) giữa chừng, hệ thống tự động hủy WebGL addon và chuyển sang Canvas.
- Nếu Canvas fail (ví dụ trên một số trình duyệt mobile cũ hoặc restricted environment), hệ thống fallback về DOM.

## Tối ưu hóa cho Mobile
Để đảm bảo trải nghiệm tốt trên thiết bị di động, BTerminal thực hiện các kỹ thuật sau:
- **Viewport Management:** Tự động điều chỉnh kích thước container ứng dụng theo `VisualViewport API`.
- **Shared PTY Dimensions (Approach C):** Giải quyết vấn đề màn hình lớn bị co lại khi thiết bị nhỏ hơn kết nối vào. 
    - Server duy trì danh sách kích thước mong muốn của tất cả các client.
    - PTY luôn được thiết lập theo kích thước **lớn nhất** (MAX rows/cols) từng được yêu cầu bởi nhóm client hiện tại.
    - Các thiết bị nhỏ hơn sẽ hiển thị terminal buffer lớn thông qua thanh cuộn (`overflow: auto`) thay vì ép PTY phải co lại.
- **Smart Cursor Follow:** Trên mobile, giao diện tự động smooth-scroll theo vị trí con trỏ khi người dùng nhập văn bản, đảm bảo vùng làm việc luôn hiển thị ngay cả khi buffer lớn hơn màn hình.
- **Sticky Modifiers:** Giải quyết vấn đề gõ tổ hợp phím trên mobile. Khi nhấn Ctrl/Alt trên màn hình, ứng dụng sẽ chuyển sang trạng thái "active" và đợi phím gõ tiếp theo từ bàn phím hệ thống (vd: nhấn Ctrl rồi gõ 'c' sẽ gửi mã byte `\x03`).
- **Input Focus Preservation:** Sử dụng `event.preventDefault()` trên sự kiện `onmousedown` của các nút ảo để ngăn trình duyệt chuyển focus khỏi terminal, giúp bàn phím hệ thống luôn mở khi người dùng thao tác with phím bổ trợ.
- **Vietnamese IME Support:** Giải quyết triệt để vấn đề không thể gõ tiếng Việt (Telex/VNI) trên mobile. 
    - **Cơ chế chặn (Blocking Logic):** Sử dụng cờ `isComposing` để tạm dừng việc gửi dữ liệu lên backend khi trình duyệt đang trong quá trình soạn ký tự (IME composition). Điều này ngăn chặn việc gửi các ký tự trung gian (ví dụ: 'a', 'a' trước khi thành 'â') gây ra hiện tượng lặp phím.
    - **Gắn listener bền bỉ (Robust Attachment):** Sử dụng cơ chế `waitForTextarea` với khả năng retry để đảm bảo các sự kiện `compositionstart` và `compositionend` luôn được gắn vào textarea ẩn của Xterm.js, ngay cả khi quá trình khởi tạo DOM bị chậm.
    - **Cấu hình Input:** Tắt các thuộc tính gây xung đột như `autocorrect`, `spellcheck` và đặt `inputmode="text"` để bộ gõ hệ thống (Gboard, iOS Keyboard) hoạt động chính xác nhất.
- **Render Refresh:** Ép buộc Xterm.js thực hiện render cycle (`term.refresh()`) và cuộn xuống cuối (`term.scrollToBottom()`) khi nhận dữ liệu mới để tránh lỗi màn hình đen trên một số trình duyệt di động.
- **Mobile Touch Selection:** Hỗ trợ cử chỉ nhấn giữ (Long Press - 500ms) để kích hoạt chế độ chọn văn bản trên thiết bị di động. Do terminal sử dụng Canvas/WebGL không có DOM text node thực, hệ thống sử dụng một lớp "Overlay" ẩn để giả lập vùng chọn, cho phép người dùng bôi đen và copy văn bản tự nhiên như ứng dụng native.

## Deployment
- Ứng dụng được đóng gói dưới dạng **Single Binary**.
- Frontend (HTML/JS/CSS) và toàn bộ tài nguyên (Fonts, Libraries) được nhúng trực tiếp vào binary Rust bằng `rust-embed`.
- Dự án không yêu cầu kết nối Internet khi khởi chạy (Zero external dependencies at runtime).

## Testing Architecture

BTerminal employs a strictly isolated testing strategy to ensure reliability and parallelism:

1.  **Unit & Integration Tests**: Backend logic (PTY management, session monitoring) is tested using standard `cargo test`. Integration tests verify the dynamic port binding logic by simulating collisions and environment overrides.
2.  **E2E Isolation (Worker-Scoped)**: 
    - Playwright is configured to run tests in parallel.
    - Each test worker utilizes a custom fixture (`e2e/fixtures.ts`) that spawns a dedicated BTerminal backend instance.
    - Servers are bound to a random port (`PORT=0`) provided by the OS, preventing conflict between parallel workers.
    - This architecture ensures that each test suite starts with a fresh `SessionRegistry`, preventing state leakage and making tests deterministic.
