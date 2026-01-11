use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use crate::pty_manager::PtyManager;

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
}

pub struct SessionRegistry {
    sessions: Arc<Mutex<std::collections::HashMap<String, Session>>>,
}

impl SessionRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub fn create_session(&self, id: String) -> Session {
        let pty_manager = Arc::new(PtyManager::new());
        let (tx, _) = broadcast::channel(100);
        let history = Arc::new(Mutex::new(Vec::new()));

        let session = Session {
            id: id.clone(),
            pty_manager: pty_manager.clone(),
            broadcast_tx: tx.clone(),
            history: history.clone(),
        };

        let tx = tx.clone();
        let history = history.clone();
        let rx = tx.subscribe();

        // Khởi động PTY reader thread
        pty_manager.start_reader(tx.clone());

        // Khởi động luồng giám sát session (lưu lịch sử và tự dọn dẹp)
        tokio::spawn(monitor_session(
            rx,
            history.clone(),
            Arc::clone(&self.sessions),
            id.clone(),
        ));

        self.sessions.lock().unwrap().insert(id, session.clone());
        session
    }

    pub fn get_session(&self, id: &str) -> Option<Session> {
        self.sessions.lock().unwrap().get(id).cloned()
    }

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

    #[tokio::test]
    async fn test_monitor_session_history_and_cleanup() {
        let (tx, rx) = broadcast::channel(10);
        let history = Arc::new(Mutex::new(Vec::new()));
        let sessions = Arc::new(Mutex::new(std::collections::HashMap::new()));
        let session_id = "test-session".to_string();

        // Giả lập session trong registry
        let pty_manager = Arc::new(PtyManager::new());
        let session = Session {
            id: session_id.clone(),
            pty_manager,
            broadcast_tx: tx.clone(),
            history: history.clone(),
        };
        sessions.lock().unwrap().insert(session_id.clone(), session);

        // Chạy monitor_session
        tokio::spawn(monitor_session(
            rx,
            history.clone(),
            sessions.clone(),
            session_id.clone(),
        ));

        // Gửi dữ liệu
        tx.send(b"hello".to_vec()).unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Kiểm tra history
        {
            let h = history.lock().unwrap();
            assert_eq!(h.as_slice(), b"hello");
        }

        // Gửi tín hiệu kết thúc
        tx.send(Vec::new()).unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Kiểm tra dọn dẹp registry
        {
            let s = sessions.lock().unwrap();
            assert!(s.get(&session_id).is_none(), "Session should be removed after empty signal");
        }
    }

    #[tokio::test]
    async fn test_session_registry_methods() {
        let registry = SessionRegistry::default();
        let session_id = "test-reg".to_string();
        
        // Create
        registry.create_session(session_id.clone());
        
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
    async fn test_monitor_session_lagged() {
        let (tx, rx) = broadcast::channel(1); // Small buffer to force lag
        let history = Arc::new(Mutex::new(Vec::new()));
        let sessions = Arc::new(Mutex::new(std::collections::HashMap::new()));
        
        tokio::spawn(monitor_session(
            rx,
            history.clone(),
            sessions.clone(),
            "lag-test".to_string(),
        ));

        // Overflow the buffer
        tx.send(b"1".to_vec()).unwrap();
        tx.send(b"2".to_vec()).unwrap();
        tx.send(b"3".to_vec()).unwrap();
        
        tokio::time::sleep(Duration::from_millis(50)).await;
        // Should have skipped some but survived
        assert!(history.lock().unwrap().len() > 0);
    }
}

/// Hàm giám sát session: lưu trữ lịch sử output và tự động xóa session khỏi registry khi PTY kết thúc.
async fn monitor_session(
    mut rx: broadcast::Receiver<Vec<u8>>,
    history: Arc<Mutex<Vec<u8>>>,
    registry_sessions: Arc<Mutex<std::collections::HashMap<String, Session>>>,
    session_id: String,
) {
    loop {
        match rx.recv().await {
            Ok(data) => {
                if data.is_empty() {
                    // PTY kết thúc, xóa session khỏi registry
                    registry_sessions.lock().unwrap().remove(&session_id);
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

impl Default for SessionRegistry {
    fn default() -> Self {
        Self::new()
    }
}
