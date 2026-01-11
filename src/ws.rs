use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use crate::session::SessionRegistry;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "type", content = "data")]
enum ClientMessage {
    Input(String),
    Resize { rows: u16, cols: u16 },
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(session_id): Path<String>,
    State(registry): State<Arc<SessionRegistry>>,
) -> impl IntoResponse {
    println!("WebSocket connection request for session: {}", session_id);
    let session = registry.get_session(&session_id);
    match session {
        Some(s) => {
            println!("Joining session: {}", session_id);
            ws.on_upgrade(move |socket| handle_socket(socket, s))
        },
        None => {
            println!("Session not found: {}", session_id);
            "Session not found".into_response()
        },
    }
}

async fn handle_socket(socket: WebSocket, session: crate::session::Session) {
    let (mut sender, mut receiver) = socket.split();

    // Send existing buffer first
    let initial_data = {
        let buffer = session.buffer.lock().unwrap();
        if !buffer.is_empty() {
            Some(buffer.clone())
        } else {
            None
        }
    };

    if let Some(data) = initial_data {
        if let Err(e) = sender.send(Message::Binary(data.into())).await {
            eprintln!("Error sending buffer: {}", e);
            return;
        }
    }

    let mut rx = session.pty.tx.subscribe();
    let pty = session.pty.clone();

    // Task for sending PTY output to WS
    let mut send_task = tokio::spawn(async move {
        while let Ok(data) = rx.recv().await {
            if let Err(e) = sender.send(Message::Binary(data.into())).await {
                eprintln!("WS send error: {}", e);
                break;
            }
        }
    });

    // Task for receiving WS input and writing to PTY
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(cmd) = serde_json::from_str::<ClientMessage>(&text) {
                        match cmd {
                            ClientMessage::Input(data) => {
                                let _ = pty.write(data.as_bytes());
                            }
                            ClientMessage::Resize { rows, cols } => {
                                let _ = pty.resize(rows, cols);
                            }
                        }
                    }
                }
                Message::Binary(data) => {
                    let _ = pty.write(&data);
                }
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}
