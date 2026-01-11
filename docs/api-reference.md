# API Reference

BTerminal sử dụng kết hợp REST API để quản lý phiên và WebSockets để truyền dữ liệu terminal.

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

#### Client Messages (JSON)
- **Input:** Gửi dữ liệu phím bấm tới terminal.
  ```json
  {"type": "Input", "data": "ls -la\n"}
  ```
- **Resize:** Cập nhật kích thước hàng/cột của PTY.
  ```json
  {"type": "Resize", "data": {"rows": 24, "cols": 80}}
  ```

#### Server Messages (Binary)
- Dữ liệu thô (raw bytes) từ PTY output để Xterm.js render.

