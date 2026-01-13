# API Reference

BTerminal sử dụng kết hợp REST API để quản lý phiên và WebSockets để truyền dữ liệu terminal.

## Server Initialization

The BTerminal server uses a dynamic port selection strategy:

1.  **Environment Variable**: Uses the value of `PORT` if defined.
2.  **Default Port**: Falls back to port `3000`.
3.  **Automatic Fallback**: If the above ports are in use, the system automatically binds to any available port (assigned by the OS).

The actual bound address is printed to stdout upon successful startup (e.g., `🚀 BTerminal is running on http://localhost:45937`).

## REST API

> **Authentication Required**: All endpoints below (except `/api/auth/*`) require a valid session cookie. Requests without authentication will return `401 Unauthorized`.

### Auth Endpoints

#### POST `/api/auth/login`
Đăng nhập vào hệ thống.
- **Request Body:** `{"username": "admin", "password": "..."}`
- **Response (200):** Thông tin user (JSON). Đặt `set-cookie` trong header.
- **Response (401):** Sai thông tin đăng nhập.

#### POST `/api/auth/logout`
Đăng xuất và hủy session.
- **Response (200):** "Logged out"

#### GET `/api/auth/me`
Lấy thông tin của user hiện tại dựa trên session cookie.
- **Response (200):** `{"id": 1, "username": "admin", "role": "admin", "must_change_password": false}`
- **Response (401):** Chưa đăng nhập.

#### POST `/api/auth/change-password`
Cập nhật mật khẩu cho user hiện tại và reset cờ ép đổi mật khẩu.
- **Request Body:** `{"new_password": "..."}`
- **Response (200):** Thông tin user sau khi cập nhật (JSON).
- **Response (401):** Unauthorized.
- **Response (500):** Lỗi database hoặc hashing.

### Session Management

### GET `/api/sessions`
Liệt kê tất cả các phiên làm việc hiện đang hoạt động.
- **Response (200):** `[{"id": "work"}, {"id": "test"}]`
- **Response (401):** Unauthorized.
- **Response (403):** Forbidden (Yêu cầu đổi mật khẩu trước).

### GET `/api/events` (SSE)
Stream các sự kiện thời gian thực tới Dashboard để cập nhật giao diện mà không cần refresh.
- **Event Data (JSON):**
  ```json
  {"type": "SessionCreated", "data": "session-id"}
  {"type": "SessionDeleted", "data": "session-id"}
  ```

---

## WebSockets

### Endpoint: `/ws/{session_id}`
Kết nối vào luồng dữ liệu thời gian thực của một session.

#### Connection Lifecycle
1.  **History Transmission**: Ngay khi kết nối thành công, server sẽ gửi toàn bộ lịch sử buffer hiện có (lên đến 100KB) dưới dạng **Binary Messages**.
2.  **Dimension Handshake**: Ngay sau lịch sử, server gửi một thông báo **SetSize** (`Text Message`) chứa kích thước PTY hiện tại để client cấu hình giao diện `xterm.js` khớp với backend.
3.  **Real-time Streaming**: Sau khi hoàn tất bắt tay trạng thái ban đầu, dữ liệu từ PTY được stream trực tiếp dưới dạng **Binary Messages**.
4.  **Graceful Exit**: Khi tiến trình shell kết thúc, server gửi một tin nhắn JSON `{"type": "Exit"}` trước khi đóng kết nối WebSocket.

#### Client Messages (JSON)
- **Input**: Gửi dữ liệu phím bấm tới terminal.
  ```json
  {"type": "Input", "data": "ls -la\n"}
  ```
- **Resize**: Cập nhật kích thước hàng/cột của PTY.
  ```json
  {"type": "Resize", "data": {"rows": 30, "cols": 100}}
  ```

#### Server Messages
- **Binary**: Dữ liệu thô (raw bytes) từ PTY output hoặc lịch sử buffer.
- **Text (JSON)**: Thông báo trạng thái hoặc điều khiển.
  - **Exit**: Khi session kết thúc.
    ```json
    {"type": "Exit"}
    ```
  - **SetSize**: Đồng bộ kích thước PTY nhỏ nhất cho tất cả client để đảm bảo không mất chữ.
    ```json
    {"type": "SetSize", "data": {"rows": 24, "cols": 80}}
    ```

---

## Technical Specifications

### PTY Environment
Every PTY process is initialized with the following environment variables to ensure consistent behavior across different host systems:
- `TERM`: `xterm-256color`
- `COLORTERM`: `truecolor`
- `LANG`: `en_US.UTF-8` (falls back to `C.UTF-8` if unavailable)

### Character Encoding
- **Encoding**: UTF-8 (Strict)
- **Special Keys**: Support for ANSI escape sequences for arrows, functional keys, and modifier combinations (Ctrl, Alt).
- **Isolation**: Data streams are strictly isolated per session ID.


