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
   - PTY Process tạo output bytes.
   - Một luồng (thread) đọc output và gửi vào `tokio::sync::broadcast` channel của session.
   - Hàm `monitor_session` lắng nghe channel này để:
     - Cập nhật buffer lịch sử (100KB gần nhất).
     - Phát hiện **Termination Signal** (vector rỗng) và:
       - Tự động xóa session khỏi `SessionRegistry`.
       - Phát một sự kiện `SessionDeleted` tới **Global Broadcast Channel** để thông báo cho toàn bộ các Dashboard đang mở (qua SSE).
   - `ws_handler` nhận tín hiệu này qua channel, thoát vòng lặp binary và gửi gói tin JSON `{"type": "Exit"}` tới Browser trước khi đóng socket.
2. **Client Input -> PTY:**
   ...
3. **Terminal Resizing:**
   ...
4. **Real-time Sync (Dashboard):**
   - Khi một session được tạo hoặc xóa, Server gửi một sự kiện thông qua `tokio::sync::broadcast` channel toàn cục.
   - Endpoint `/api/events` (Server-Sent Events) lắng nghe channel này và đẩy thông báo tới toàn bộ Dashboard đang mở.
   - Browser tự động cập nhật danh sách session hoặc hiển thị thông báo nếu session hiện tại bị xóa, đảm bảo tính nhất quán dữ liệu giữa nhiều thiết bị.

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
- **Viewport Management:** Tự động điều chỉnh kích thước PTY khi bàn phím ảo xuất hiện/biến mất thông qua `VisualViewport API`. Thay vì chỉ dùng `ResizeObserver`, ứng dụng lắng nghe sự kiện `resize` của viewport thực tế để điều chỉnh trực tiếp chiều cao của container ứng dụng (`#app`) theo `viewport.height`. Cách tiếp cận này đảm bảo Xterm.js luôn nhận biết chính xác vùng hiển thị thực tế, tự động cuộn con trỏ vào view và đẩy thanh phím ảo lên trên bàn phím hệ thống một cách tự nhiên theo luồng layout của trình duyệt.
- **Sticky Modifiers:** Giải quyết vấn đề gõ tổ hợp phím trên mobile. Khi nhấn Ctrl/Alt trên màn hình, ứng dụng sẽ chuyển sang trạng thái "active" và đợi phím gõ tiếp theo từ bàn phím hệ thống (vd: nhấn Ctrl rồi gõ 'c' sẽ gửi mã byte `\x03`).
- **Input Focus Preservation:** Sử dụng `event.preventDefault()` trên sự kiện `onmousedown` của các nút ảo để ngăn trình duyệt chuyển focus khỏi terminal, giúp bàn phím hệ thống luôn mở khi người dùng thao tác with phím bổ trợ.
- **Vietnamese IME Support:** Giải quyết vấn đề không thể gõ tiếng Việt (Telex/VNI) trên mobile. Để tránh hiện tượng lặp phím (duplicate characters), ứng dụng tắt `screenReaderMode` của Xterm.js trên mobile và cấu hình các thuộc tính của hidden textarea (`autocorrect="off"`, `spellcheck="false"`, `inputmode="text"`). Cấu hình này ngăn chặn trình duyệt thực hiện các hiệu chỉnh văn bản tự động gây xung đột với quá trình xử lý ký tự (composition) của bàn phím hệ thống.
- **Render Refresh:** Ép buộc Xterm.js thực hiện render cycle (`term.refresh()`) và cuộn xuống cuối (`term.scrollToBottom()`) khi nhận dữ liệu mới để tránh lỗi màn hình đen trên một số trình duyệt di động.

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
