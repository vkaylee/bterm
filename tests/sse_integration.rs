use std::sync::Arc;
use tokio::net::TcpListener;
use bterminal::session::SessionRegistry;
use bterminal::AppState;
use axum::{routing::get, Router};
use bterminal::api::{events_handler, create_session, CreateSessionRequest};
use tokio::sync::broadcast;
use futures_util::StreamExt;
use axum::extract::{State, Json};

#[tokio::test]
async fn test_sse_events_flow() {
    let (tx, _) = broadcast::channel(10);
    let registry = Arc::new(SessionRegistry::new(tx.clone()));
    let state = Arc::new(AppState { registry: registry.clone(), tx: tx.clone() });

    let app = Router::new()
        .route("/api/events", get(events_handler))
        .with_state(state.clone());

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // 1. Connect to SSE
    let client = reqwest::Client::new();
    let url = format!("http://{}:{}/api/events", addr.ip(), addr.port());
    
    let mut stream = client.get(url)
        .send()
        .await
        .unwrap()
        .bytes_stream();

    // 2. Trigger an event (SessionCreated)
    let payload = Json(CreateSessionRequest { id: "sse-test".to_string() });
    create_session(State(state), payload).await;

    // 3. Verify event received in SSE stream
    let first_chunk = stream.next().await.unwrap().unwrap();
    let chunk_str = String::from_utf8_lossy(&first_chunk);
    
    // SSE format is "data: <json>\n\n"
    assert!(chunk_str.contains("data:"));
    assert!(chunk_str.contains("SessionCreated"));
    assert!(chunk_str.contains("sse-test"));
}

#[tokio::test]
async fn test_sse_automatic_exit_flow() {
    let (tx, _) = broadcast::channel(10);
    let registry = Arc::new(SessionRegistry::new(tx.clone()));
    let state = Arc::new(AppState { registry: registry.clone(), tx: tx.clone() });

    let app = Router::new()
        .route("/api/events", get(events_handler))
        .with_state(state.clone());

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // 1. Connect to SSE
    let client = reqwest::Client::new();
    let url = format!("http://{addr}/api/events");
    let mut stream = client.get(url).send().await.unwrap().bytes_stream();

    // 2. Create a session (triggers monitor_session)
    let session = registry.create_session("auto-exit-test".to_string());
    
    // 3. Simulate PTY termination
    session.broadcast_tx.send(Vec::new()).unwrap();

    // 4. Verify we eventually get a SessionDeleted event
    let mut found_deleted = false;
    let timeout = tokio::time::sleep(std::time::Duration::from_secs(5));
    tokio::pin!(timeout);

    loop {
        tokio::select! {
            chunk = stream.next() => {
                if let Some(Ok(bytes)) = chunk {
                    let s = String::from_utf8_lossy(&bytes);
                    if s.contains("SessionDeleted") && s.contains("auto-exit-test") {
                        found_deleted = true;
                        break;
                    }
                } else {
                    break;
                }
            }
            _ = &mut timeout => break,
        }
    }
    
    assert!(found_deleted, "Should have received SessionDeleted event via SSE");
}
