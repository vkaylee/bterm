# API Reference

BTerminal sử dụng kết hợp REST API để quản lý phiên và WebSockets để truyền dữ liệu terminal.

## Server Initialization

The BTerminal server uses a dynamic port selection strategy:

1.  **Environment Variable**: Uses the value of `PORT` if defined.
2.  **Default Port**: Falls back to port `3000`.
3.  **Automatic Fallback**: If the above ports are in use, the system automatically binds to any available port (assigned by the OS).

The actual bound address is printed to stdout upon successful startup (e.g., `🚀 BTerminal is running on http://localhost:45937`).

## REST API

### GET `/api/sessions`
Liệt kê tất cả các phiên làm việc hiện đang hoạt động.
- **Response (200):** `[{"id": "work"}, {"id": "test"}]`

### POST `/api/sessions`
Tạo một phiên làm việc mới.
- **Request Body:** `{"id": "string"}`
- **Response (200):** `"Created"`

### DELETE `/api/sessions/{id}`
Xóa một phiên làm việc và đóng PTY process liên quan.
- **Response (200):** OK
- **Response (404):** Not Found

---

## WebSockets

### Endpoint: `/ws/{session_id}`
Kết nối vào luồng dữ liệu thời gian thực của một session.

#### Connection Lifecycle
1.  **History Transmission**: Ngay khi kết nối thành công, server sẽ gửi toàn bộ lịch sử buffer hiện có (lên đến 100KB) dưới dạng **Binary Messages** trước khi bắt đầu truyền dữ liệu thời gian thực.
2.  **Real-time Streaming**: Dữ liệu từ PTY được truyền dưới dạng **Binary Messages**.
3.  **Graceful Exit**: Khi tiến trình shell kết thúc, server gửi một tin nhắn JSON `{"type": "Exit"}` trước khi đóng kết nối WebSocket.

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
- **Text (JSON)**: Thông báo trạng thái (ví dụ: `{"type": "Exit"}`).


