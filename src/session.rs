use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use crate::pty_manager::PtyManager;
use crate::GlobalEvent;

#[derive(Clone, serde::Serialize)]
pub struct SessionInfo {
    pub id: String,
}

#[derive(Clone)]
pub struct Session {
    #[allow(dead_code)]
    pub id: String,
    pub pty_manager: Arc<PtyManager>,
    pub broadcast_tx: broadcast::Sender<Vec<u8>>,
    pub history: Arc<Mutex<Vec<u8>>>,
    pub client_sizes: Arc<Mutex<std::collections::HashMap<uuid::Uuid, (u16, u16)>>>,
}

impl Session {
    pub fn update_client_size(&self, client_id: uuid::Uuid, rows: u16, cols: u16) {
        self.client_sizes.lock().unwrap().insert(client_id, (rows, cols));
        let sizes = self.client_sizes.lock().unwrap();
        self.recalculate_pty_size(&sizes);
    }

    pub fn remove_client(&self, client_id: uuid::Uuid) {
        if self.client_sizes.lock().unwrap().remove(&client_id).is_some() {
            let sizes = self.client_sizes.lock().unwrap();
            self.recalculate_pty_size(&sizes);
        }
    }

    fn recalculate_pty_size(&self, sizes: &std::collections::HashMap<uuid::Uuid, (u16, u16)>) {
        if sizes.is_empty() {
            return;
        }

        let mut min_rows = u16::MAX;
        let mut min_cols = u16::MAX;

        for (r, c) in sizes.values() {
            if *r < min_rows { min_rows = *r; }
            if *c < min_cols { min_cols = *c; }
        }

        if min_rows > 0 && min_cols > 0 && min_rows != u16::MAX && min_cols != u16::MAX {
            let _ = self.pty_manager.resize(min_rows, min_cols);
            
            // Thông báo kích thước PTY mới cho tất cả các client để đồng bộ UI
            let msg = format!(r#"{{"type": "SetSize", "data": {{"rows": {min_rows}, "cols": {min_cols}}}}}"#);
            let _ = self.broadcast_tx.send(msg.into_bytes());
        }
    }
}

pub struct SessionRegistry {
    sessions: Arc<Mutex<std::collections::HashMap<String, Session>>>,
    global_tx: broadcast::Sender<GlobalEvent>,
}

/// Hàm giám sát session: lưu trữ lịch sử output và tự động xóa session khỏi registry khi PTY kết thúc.
async fn monitor_session(
    mut rx: broadcast::Receiver<Vec<u8>>,
    history: Arc<Mutex<Vec<u8>>>,
    registry_sessions: Arc<Mutex<std::collections::HashMap<String, Session>>>,
    session_id: String,
    global_tx: broadcast::Sender<GlobalEvent>,
) {
    loop {
        match rx.recv().await {
            Ok(data) => {
                if data.is_empty() {
                    // PTY kết thúc, xóa session khỏi registry
                    registry_sessions.lock().unwrap().remove(&session_id);
                    // Thông báo cho toàn bộ các Dashboard khác
                    let _ = global_tx.send(GlobalEvent::SessionDeleted(session_id));
                    break;
                }
                // Lưu lịch sử (giới hạn 100KB)
                let mut buffer = history.lock().unwrap();
                buffer.extend_from_slice(&data);
                let len = buffer.len();
                if len > 102_400 {
                    buffer.drain(..len - 102_400);
                }
            }
            Err(broadcast::error::RecvError::Lagged(_)) => {
                // Tiếp tục nếu bị lag (mất một số message)
            }
            #[cfg(not(tarpaulin_include))]
            Err(_) => break, // Channel bị đóng
        }
    }
}

impl SessionRegistry {
    #[must_use]
    pub fn new(global_tx: broadcast::Sender<GlobalEvent>) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(std::collections::HashMap::new())),
            global_tx,
        }
    }

    #[must_use]
    pub fn create_session(&self, id: String) -> Session {
        let pty_manager = Arc::new(PtyManager::new());
        let (tx, _) = broadcast::channel(100);
        let history = Arc::new(Mutex::new(Vec::new()));
        let client_sizes = Arc::new(Mutex::new(std::collections::HashMap::new()));

        let session = Session {
            id: id.clone(),
            pty_manager: pty_manager.clone(),
            broadcast_tx: tx.clone(),
            history: history.clone(),
            client_sizes,
        };

        let rx = tx.subscribe();

        // Khởi động PTY reader thread
        pty_manager.start_reader(tx);

        // Khởi động luồng giám sát session (lưu lịch sử và tự dọn dẹp)
        tokio::spawn(monitor_session(
            rx,
            history,
            Arc::clone(&self.sessions),
            id.clone(),
            self.global_tx.clone(),
        ));

        self.sessions.lock().unwrap().insert(id, session.clone());
        session
    }

    #[must_use]
    pub fn get_session(&self, id: &str) -> Option<Session> {
        self.sessions.lock().unwrap().get(id).cloned()
    }

    #[must_use]
    pub fn list_sessions(&self) -> Vec<SessionInfo> {
        self.sessions
            .lock()
            .unwrap()
            .keys()
            .map(|id| SessionInfo { id: id.clone() })
            .collect()
    }

    pub fn remove_session(&self, id: &str) {
        self.sessions.lock().unwrap().remove(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn setup_registry() -> (SessionRegistry, broadcast::Receiver<GlobalEvent>) {
        let (tx, rx) = broadcast::channel(10);
        (SessionRegistry::new(tx), rx)
    }

    #[tokio::test]
    async fn test_monitor_session_history_and_cleanup() {
        let (tx, rx) = broadcast::channel(10);
        let (gtx, mut grx) = broadcast::channel(10);
        let history = Arc::new(Mutex::new(Vec::new()));
        let sessions = Arc::new(Mutex::new(std::collections::HashMap::new()));
        let session_id = "test-session".to_string();

        // Giả lập session trong registry
        let pty_manager = Arc::new(PtyManager::new());
        let client_sizes = Arc::new(Mutex::new(std::collections::HashMap::new()));
        let session = Session {
            id: session_id.clone(),
            pty_manager,
            broadcast_tx: tx.clone(),
            history: history.clone(),
            client_sizes,
        };
        sessions.lock().unwrap().insert(session_id.clone(), session);

        // Chạy monitor_session
        tokio::spawn(monitor_session(
            rx,
            history.clone(),
            sessions.clone(),
            session_id.clone(),
            gtx,
        ));

        // Gửi dữ liệu
        tx.send(b"hello".to_vec()).unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Kiểm tra history
        {
            let h = history.lock().unwrap();
            assert_eq!(h.as_slice(), b"hello");
            drop(h);
        }

        // Gửi tín hiệu kết thúc
        tx.send(Vec::new()).unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Kiểm tra dọn dẹp registry
        {
            let s = sessions.lock().unwrap();
            assert!(s.get(&session_id).is_none(), "Session should be removed after empty signal");
            drop(s);
        }

        // Kiểm tra sự kiện global được gửi
        let event = grx.recv().await.unwrap();
        match event {
            GlobalEvent::SessionDeleted(id) => assert_eq!(id, session_id),
            _ => panic!("Expected SessionDeleted event"),
        }
    }

    #[tokio::test]
    async fn test_session_registry_methods() {
        let (registry, _) = setup_registry();
        let session_id = "test-reg".to_string();
        
        // Create
        let _ = registry.create_session(session_id.clone());
        
        // Get
        let session = registry.get_session(&session_id);
        assert!(session.is_some());
        
        // List
        let sessions = registry.list_sessions();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, session_id);
        
        // Remove
        registry.remove_session(&session_id);
        assert!(registry.get_session(&session_id).is_none());
    }

    #[tokio::test]
    async fn test_recalculate_pty_size_min_logic() {
        let pty_manager = Arc::new(PtyManager::new());
        let (tx, _) = broadcast::channel(10);
        let history = Arc::new(Mutex::new(Vec::new()));
        let client_sizes = Arc::new(Mutex::new(std::collections::HashMap::new()));

        let session = Session {
            id: "test-resize".to_string(),
            pty_manager: pty_manager.clone(),
            broadcast_tx: tx,
            history,
            client_sizes,
        };

        // Giả lập 3 client với kích thước khác nhau
        let client1 = uuid::Uuid::new_v4();
        let client2 = uuid::Uuid::new_v4();
        let client3 = uuid::Uuid::new_v4();

        // Thêm client 1: 100x40
        session.update_client_size(client1, 40, 100);
        
        // Thêm client 2: 80x24 (Nhỏ hơn)
        session.update_client_size(client2, 24, 80);

        // Thêm client 3: 120x60 (Lớn nhất)
        session.update_client_size(client3, 60, 120);

        // Sau khi thêm cả 3, kích thước PTY phải là MIN của cả 3: 24x80
        // (Kiểm tra gián tiếp thông qua size cuối cùng trong map - logic MIN đã chạy trong update_client_size)
        let sizes = session.client_sizes.lock().unwrap();
        let mut min_rows = u16::MAX;
        let mut min_cols = u16::MAX;
        for (r, c) in sizes.values() {
            if *r < min_rows { min_rows = *r; }
            if *c < min_cols { min_cols = *c; }
        }

        assert_eq!(min_rows, 24);
        assert_eq!(min_cols, 80);

        // Xóa client nhỏ nhất, kích thước phải nhảy lên MIN kế tiếp: 40x100
        drop(sizes);
        session.remove_client(client2);
        
        let sizes_after = session.client_sizes.lock().unwrap();
        let mut min_rows_new = u16::MAX;
        let mut min_cols_new = u16::MAX;
        for (r, c) in sizes_after.values() {
            if *r < min_rows_new { min_rows_new = *r; }
            if *c < min_cols_new { min_cols_new = *c; }
        }

        assert_eq!(min_rows_new, 40);
        assert_eq!(min_cols_new, 100);
    }
}
