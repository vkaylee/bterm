use ax_ws::{Message, WebSocket, WebSocketUpgrade};
use axum::{
    extract::{ws as ax_ws, State},
    response::IntoResponse,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use serde::Deserialize;
use crate::session::Session;
use crate::AppState;

#[derive(Deserialize)]
#[serde(tag = "type", content = "data")]
enum ClientMessage {
    Input(String),
    Resize { rows: u16, cols: u16 },
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    axum::extract::Path(session_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let session = state.registry.get_session(&session_id);

    session.map_or_else(|| {
        println!("Session not found: {session_id}");
        "Session not found".into_response()
    }, |s| {
        println!("Joining session: {session_id}");
        ws.on_upgrade(move |socket| handle_socket(socket, s))
    })
}

async fn handle_socket(socket: WebSocket, session: Session) {
    let (mut sender, mut receiver) = socket.split();

    // Send history first
    let history_data = {
        let history = session.history.lock().unwrap();
        if history.is_empty() {
            None
        } else {
            Some(history.clone())
        }
    };

    if let Some(data) = history_data {
        if let Err(e) = sender.send(Message::Binary(data.into())).await {
            #[cfg(not(tarpaulin_include))]
            println!("Error sending history: {e}");
            return;
        }
    }

    let mut rx = session.broadcast_tx.subscribe();
    let pty = session.pty_manager.clone();

    // Spawn a task to forward PTY output to WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Ok(data) = rx.recv().await {
            if data.is_empty() {
                break; // PTY ended
            }
            let bin_data: Vec<u8> = data;
            if let Err(e) = sender.send(Message::Binary(bin_data.into())).await {
                println!("WS send error: {e}");
                return;
            }
        }
        
        // If we reach here, it means the broadcast channel is closed or we got an empty signal
        let _ = sender.send(Message::Text(r#"{"type": "Exit"}"#.into())).await;
    });

    // Handle incoming messages from WebSocket
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                    match client_msg {
                        ClientMessage::Input(data) => {
                            if let Err(e) = pty.write(data.as_bytes()) {
                                #[cfg(not(tarpaulin_include))]
                                println!("PTY write error: {e}");
                            }
                        }
                        ClientMessage::Resize { rows, cols } => {
                            if let Err(e) = pty.resize(rows, cols) {
                                #[cfg(not(tarpaulin_include))]
                                println!("PTY resize error: {e}");
                            }
                        }
                    }
                }
            }
        }
    });

    // If either task fails, abort the other
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::{routing::get, Router};
    use crate::session::SessionRegistry;
    use tokio::sync::broadcast;

    fn setup_state() -> Arc<AppState> {
        let registry = Arc::new(SessionRegistry::new());
        let (tx, _) = broadcast::channel(10);
        Arc::new(AppState { registry, tx })
    }

    #[test]
    fn test_client_message_deserialization() {
        let input_json = r#"{"type": "Input", "data": "ls\n"}"#;
        let msg: ClientMessage = serde_json::from_str(input_json).unwrap();
        match msg {
            ClientMessage::Input(data) => assert_eq!(data, "ls\n"),
            _ => panic!("Expected Input"),
        }

        let resize_json = r#"{"type": "Resize", "data": {"rows": 24, "cols": 80}}"#;
        let msg: ClientMessage = serde_json::from_str(resize_json).unwrap();
        match msg {
            ClientMessage::Resize { rows, cols } => {
                assert_eq!(rows, 24);
                assert_eq!(cols, 80);
            }
            _ => panic!("Expected Resize"),
        }
    }

    #[tokio::test]
    async fn test_ws_handler_not_found() {
        use tokio_tungstenite::connect_async;
        use tokio::net::TcpListener;

        let state = setup_state();
        let app = Router::new()
            .route("/ws/{session_id}", get(ws_handler))
            .with_state(state);

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let url = format!("ws://{}:{}/ws/invalid", addr.ip(), addr.port());
        let result = connect_async(url).await;
        match result {
            Err(tokio_tungstenite::tungstenite::Error::Http(resp)) => {
                assert_eq!(resp.status(), StatusCode::OK);
                let body = resp.into_body().unwrap();
                assert_eq!(body, "Session not found".as_bytes().to_vec());
            }
            _ => panic!("Expected HTTP error with Session not found message, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_ws_history_sent() {
        use tokio_tungstenite::connect_async;
        use tokio::net::TcpListener;

        let state = setup_state();
        let session_id = "history-test".to_string();
        let session = state.registry.create_session(session_id.clone());
        
        // Add some history
        {
            let mut history = session.history.lock().unwrap();
            history.extend_from_slice(b"old data");
        }

        let app = Router::new()
            .route("/ws/{session_id}", get(ws_handler))
            .with_state(state);

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let url = format!("ws://{}:{}/ws/{}", addr.ip(), addr.port(), session_id);
        let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
        let (_, mut read) = ws_stream.split();

        // First message should be the history
        let msg = read.next().await.unwrap().unwrap();
        assert_eq!(msg.into_data().as_ref(), b"old data");
    }
}