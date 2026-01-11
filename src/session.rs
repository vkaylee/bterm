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

        let tx_clone = tx.clone();
        let history_clone = history.clone();
        let mut rx = tx.subscribe();

        // Luồng lưu trữ lịch sử terminal (100KB)
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(data) => {
                        let mut buffer = history_clone.lock().unwrap();
                        buffer.extend_from_slice(&data);
                        let len = buffer.len();
                        if len > 102_400 {
                            buffer.drain(..len - 102_400);
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {}
                    Err(_) => break,
                }
            }
        });

        // Khởi động PTY reader thread
        pty_manager.start_reader(tx_clone);

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

impl Default for SessionRegistry {
    fn default() -> Self {
        Self::new()
    }
}
