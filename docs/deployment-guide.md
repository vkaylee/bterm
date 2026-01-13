# Deployment Guide

Tài liệu này hướng dẫn cách triển khai BTerminal lên môi trường Production.

## 📦 Hình thức phân phối

BTerminal được thiết kế dưới dạng **Single Portable Binary**. Toàn bộ tài nguyên (Frontend, Fonts, Libraries) đã được nhúng trực tiếp vào file thực thi. Bạn không cần cài đặt Node.js hay các thư viện runtime khác trên server.

## 🛠️ Yêu cầu hệ thống

- **Hệ điều hành:** Linux (Khuyên dùng: Ubuntu 22.04+, Debian 11+). 
    - *Lưu ý: Tính năng PTY yêu cầu các thư viện hệ thống POSIX tiêu chuẩn.*
- **Quyền hạn:** User chạy ứng dụng cần có quyền tạo PTY (thông thường là quyền user tiêu chuẩn).
- **Lưu trữ:** Khoảng 10MB cho file binary và dung lượng nhỏ cho file database SQLite (`bterminal.db`).

## ⚙️ Cấu hình (Environment Variables)

Ứng dụng hỗ trợ các biến môi trường sau:

| Biến | Mô tả | Mặc định |
|------|-------|----------|
| `PORT` | Cổng dịch vụ lắng nghe | `3000` |
| `DATABASE_URL` | Đường dẫn file database SQLite | `sqlite:bterminal.db` |
| `RUST_LOG` | Cấp độ ghi log (error, info, debug) | `info` |

## 🚀 Các bước triển khai nhanh

1. **Copy binary lên server:**
   ```bash
   scp target/release/bterminal user@your-server:/usr/local/bin/
   ```

2. **Chạy ứng dụng lần đầu:**
   ```bash
   PORT=8080 bterminal
   ```

3. **Thiết lập bảo mật:**
   - Truy cập giao diện web.
   - Đăng nhập với tài khoản mặc định: `admin` / `admin`.
   - Hệ thống sẽ **bắt buộc** bạn đổi mật khẩu ngay lập tức. Hãy đặt một mật khẩu cực kỳ an toàn.

## 🛡️ Cấu hình Reverse Proxy (Khuyên dùng)

Nên sử dụng Nginx hoặc Caddy phía trước để hỗ trợ HTTPS và quản lý WebSocket ổn định.

### Ví dụ cấu hình Nginx:
```nginx
server {
    server_name your-terminal.example.com;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
```

## 🔄 Quản lý dịch vụ với Systemd

Tạo file `/etc/systemd/system/bterminal.service`:

```ini
[Unit]
Description=BTerminal Service
After=network.target

[Service]
Type=simple
User=your-user
WorkingDirectory=/home/your-user/bterminal
ExecStart=/usr/local/bin/bterminal
Restart=always
Environment=PORT=3000
Environment=DATABASE_URL=sqlite:/home/your-user/bterminal/bterminal.db

[Install]
WantedBy=multi-user.target
```

**Kích hoạt dịch vụ:**
```bash
sudo systemctl daemon-reload
sudo systemctl enable bterminal
sudo systemctl start bterminal
```

## 🧹 Bảo trì

- **Backup:** Chỉ cần sao lưu file `bterminal.db`.
- **Update:** Thay thế file binary cũ bằng file mới và khởi động lại dịch vụ qua systemd.
- **Log:** Kiểm tra log qua `journalctl -u bterminal -f`.
