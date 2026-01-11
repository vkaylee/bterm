use std::sync::Arc;
use tokio::net::TcpListener;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use bterminal::session::SessionRegistry;
use bterminal::AppState;
use axum::{routing::get, Router};
use bterminal::ws::ws_handler;
use tokio::sync::broadcast;

#[tokio::test]
async fn test_websocket_flow() {
    let registry = Arc::new(SessionRegistry::new());
    let (tx, _) = broadcast::channel(10);
    let state = Arc::new(AppState { registry: registry.clone(), tx });
    
    let session_id = "ws-test".to_string();
    registry.create_session(session_id.clone());

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
    let (mut write, mut read) = ws_stream.split();

    // 1. Send input
    let input_msg = r#"{"type": "Input", "data": "echo hello\n"}"#;
    write.send(tokio_tungstenite::tungstenite::Message::Text(input_msg.into())).await.unwrap();

    // 2. Read output (Wait for some data)
    let msg = read.next().await.unwrap().unwrap();
    assert!(msg.is_binary() || msg.is_text());

    // 3. Send resize
    let resize_msg = r#"{"type": "Resize", "data": {"rows": 30, "cols": 100}}"#;
    write.send(tokio_tungstenite::tungstenite::Message::Text(resize_msg.into())).await.unwrap();
}
