use crate::pty_manager::PtyInstance;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use portable_pty::PtySize;
use serde::{Serialize, Deserialize};

#[derive(Clone)]
pub struct Session {
    pub id: String,
    pub pty: Arc<PtyInstance>,
    pub buffer: Arc<Mutex<Vec<u8>>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SessionInfo {
    pub id: String,
}

pub struct SessionRegistry {
    pub sessions: Mutex<HashMap<String, Session>>,
}

impl SessionRegistry {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }

    pub fn create_session(&self, id: String) -> anyhow::Result<Session> {
        let pty = PtyInstance::new("/bin/bash", PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let pty_arc = Arc::new(pty);
        let buffer = Arc::new(Mutex::new(Vec::new()));

        let session = Session {
            id: id.clone(),
            pty: pty_arc.clone(),
            buffer: buffer.clone(),
        };

        // Spawn a task to update the buffer
        let mut rx = pty_arc.tx.subscribe();
        let buffer_clone = buffer.clone();
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(data) => {
                        let mut buf = buffer_clone.lock().unwrap();
                        buf.extend_from_slice(&data);
                        // Keep buffer size reasonable (e.g., 100KB)
                        if buf.len() > 100_000 {
                            let to_remove = buf.len() - 100_000;
                            buf.drain(0..to_remove);
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                }
            }
        });

        self.sessions.lock().unwrap().insert(id, session.clone());
        Ok(session)
    }

    pub fn get_session(&self, id: &str) -> Option<Session> {
        self.sessions.lock().unwrap().get(id).cloned()
    }

    pub fn delete_session(&self, id: &str) -> bool {
        self.sessions.lock().unwrap().remove(id).is_some()
    }

    pub fn list_sessions(&self) -> Vec<SessionInfo> {
        self.sessions.lock().unwrap().keys().map(|k| SessionInfo { id: k.clone() }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_lifecycle() {
        let registry = SessionRegistry::new();
        let session_id = "test-session".to_string();

        // 1. Create
        let create_res = registry.create_session(session_id.clone());
        assert!(create_res.is_ok());

        // 2. Get
        let session = registry.get_session(&session_id);
        assert!(session.is_some());
        assert_eq!(session.unwrap().id, session_id);

        // 3. List
        let list = registry.list_sessions();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, session_id);

        // 4. Delete
        let deleted = registry.delete_session(&session_id);
        assert!(deleted);
        assert!(registry.get_session(&session_id).is_none());
    }
}
