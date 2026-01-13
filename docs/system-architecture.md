# System Architecture

## Thành phần hệ thống

```ascii
┌──────────────┐      HTTP (JSON)      ┌──────────────────┐      ┌──────────────┐
│   Browser    │◄─────────────────────▶│   Axum Server    │◄────▶│   SQLite     │
│  (Dashboard) │◄─────────────────────┤ (Auth & Registry)│      │ (bterminal.db)│
└──────────────┘      SSE Events       └────────┬─────────┘      └──────────────┘
        ▲                                       │
        │ WebSockets (Binary)                   │ Spawn
        ▼                                       ▼
┌──────────────┐                       ┌──────────────────┐
│   Xterm.js   │◄─────────────────────▶│   PTY Instance   │
│  (Terminal)  │    Broadcast Channel  │ (/bin/bash etc.) │
└──────────────┘                       └──────────────────┘
```

## Luồng dữ liệu (Data Flow)
1. **Authentication (New):** Người dùng phải đăng nhập qua `/api/auth/login`. Server xác thực bằng mật khẩu băm (Argon2) và trả về Session Cookie (HttpOnly).
2. **PTY Output -> Clients:**
   ...
3. **Client Input -> PTY:**
   ...
4. **Terminal Resizing:**
   ...
5. **Real-time Sync (Dashboard):**
   ...

## Bảo mật (Security)

BTerminal triển khai hệ thống bảo mật tự quản (self-contained) để đảm bảo chỉ những người dùng được ủy quyền mới có quyền truy cập vào terminal:

- **Xác thực Cookie-based:** Sử dụng `tower-sessions` để quản lý trạng thái đăng nhập. Cookie được cấu hình với `HttpOnly` và `SameSite=Strict` để chống tấn công XSS và CSRF.
- **Mã hóa Argon2id:** Mật khẩu người dùng không bao giờ được lưu dưới dạng văn bản thô. BTerminal sử dụng Argon2id (người chiến thắng Password Hashing Competition) với muối (salt) ngẫu nhiên cho mỗi user.
- **Middleware Authorization:** Mọi request tới API quản lý session hoặc WebSocket terminal đều phải đi qua lớp middleware kiểm tra tính hợp lệ của session. Nếu không hợp lệ, hệ thống trả về mã lỗi `401 Unauthorized`.
- **Ép đổi mật khẩu (First Login):** Hệ thống theo dõi trạng thái `must_change_password` trong database. 
    - Nếu cờ này là `true` (mặc định cho user mới), middleware sẽ trả về `403 Forbidden` cho mọi request truy cập terminal.
    - Giao diện đăng nhập sẽ tự động hiển thị màn hình đổi mật khẩu và chỉ cho phép tiếp tục sau khi user đã cập nhật mật khẩu mới.
- **Database Cô lập:** Toàn bộ thông tin user và role được lưu trữ trong file SQLite (`bterminal.db`) nằm cục bộ trên server, không yêu cầu kết nối database server bên ngoài.

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
- **Shared PTY Dimensions (Smallest Screen Priority):** Giải quyết vấn đề mất chữ và mất dấu nhắc lệnh (prompt) trên thiết bị di động khi dùng chung session với máy tính.
    - Server duy trì danh sách kích thước (rows/cols) của tất cả các client đang kết nối.
    - PTY luôn được thiết lập theo kích thước **nhỏ nhất** (MIN rows/cols) trong số các client hiện tại.
    - **Lợi ích:** Đảm bảo Shell (bash/zsh) luôn ngắt dòng (wrap) chuẩn xác cho màn hình nhỏ nhất, giúp người dùng mobile luôn thấy được nội dung mới nhất mà không cần cuộn ngang.
    - **UI Centering:** Trên các màn hình lớn (Desktop), terminal sẽ được hiển thị ở chính giữa với vùng đệm (padding) xung quanh để duy trì tỉ lệ chuẩn của PTY.
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

1.  **Unit & Integration Tests**: Backend logic (PTY management, session monitoring, Auth flow) is tested using standard `cargo test`. Integration tests verify the dynamic port binding logic and SQLite persistence.
2.  **E2E Isolation (Worker-Scoped)**: 
    - Playwright is configured to run tests in parallel.
    - **Dedicated Backend**: Each test worker utilizes a custom fixture (`e2e/fixtures.ts`) that spawns a dedicated BTerminal backend instance on a random port.
    - **In-Memory Persistence**: Every worker uses its own isolated **In-Memory SQLite** database. This ensures zero state leakage between workers and eliminates database file locks.
    - **Auto-Authentication**: The test fixture automatically performs a backend login via the API and injects the session cookie into the browser context before any test code runs. This allows tests to focus on feature logic while maintaining full security coverage.
