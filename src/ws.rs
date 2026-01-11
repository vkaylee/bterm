use ax_ws::{Message, WebSocket, WebSocketUpgrade};
use axum::{
    extract::{ws as ax_ws, State},
    response::IntoResponse,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use serde::Deserialize;
use crate::session::{Session, SessionRegistry};

#[derive(Deserialize)]
#[serde(tag = "type", content = "data")]
enum ClientMessage {
    Input(String),
    Resize { rows: u16, cols: u16 },
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(registry): State<Arc<SessionRegistry>>,
    axum::extract::Path(session_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let session = registry.get_session(&session_id);

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
            println!("Error sending history: {e}");
            return;
        }
    }

    let mut rx = session.broadcast_tx.subscribe();
    let pty = session.pty_manager.clone();

    // Spawn a task to forward PTY output to WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Ok(data) = rx.recv().await {
            let bin_data: Vec<u8> = data;
            if let Err(e) = sender.send(Message::Binary(bin_data.into())).await {
                println!("WS send error: {e}");
                break;
            }
        }
    });

    // Handle incoming messages from WebSocket
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                    match client_msg {
                        ClientMessage::Input(data) => {
                            if let Err(e) = pty.write(data.as_bytes()) {
                                println!("PTY write error: {e}");
                            }
                        }
                        ClientMessage::Resize { rows, cols } => {
                            if let Err(e) = pty.resize(rows, cols) {
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
