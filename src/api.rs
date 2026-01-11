use axum::extract::{Json, State};
use axum::response::IntoResponse;
use std::sync::Arc;
use crate::session::SessionInfo;
use crate::AppState;
use serde::Deserialize;
use axum::response::sse::{Event, Sse};
use futures_util::stream::Stream;
use std::convert::Infallible;

#[derive(Deserialize)]
pub struct CreateSessionRequest {
    pub id: String,
}

pub async fn list_sessions(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<SessionInfo>> {
    println!("API: Listing sessions");
    Json(state.registry.list_sessions())
}

pub async fn create_session(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateSessionRequest>,
) -> impl IntoResponse {
    println!("API: Creating session with ID: {}", payload.id);
    let _session = state.registry.create_session(payload.id.clone());
    let _ = state.tx.send(crate::GlobalEvent::SessionCreated(payload.id));
    println!("API: Session created successfully.");
    Json("Created").into_response()
}

pub async fn delete_session(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    state.registry.remove_session(&id);
    let _ = state.tx.send(crate::GlobalEvent::SessionDeleted(id));
    axum::http::StatusCode::OK.into_response()
}

pub async fn events_handler(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut rx = state.tx.subscribe();

    let stream = async_stream::stream! {
        while let Ok(msg) = rx.recv().await {
            match serde_json::to_string(&msg) {
                Ok(data) => yield Ok(Event::default().data(data)),
                Err(_) => continue,
            }
        }
    };

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::Path;
    use crate::session::SessionRegistry;
    use tokio::sync::broadcast;

    fn setup() -> Arc<AppState> {
        let registry = Arc::new(SessionRegistry::new());
        let (tx, _) = broadcast::channel(10);
        Arc::new(AppState { registry, tx })
    }

    #[tokio::test]
    async fn test_create_and_list_sessions() {
        let state = setup();
        
        // Create
        let req = Json(CreateSessionRequest { id: "test-id".to_string() });
        create_session(State(state.clone()), req).await;
        
        // List
        let Json(sessions) = list_sessions(State(state.clone())).await;
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, "test-id");
    }

    #[tokio::test]
    async fn test_delete_session() {
        let state = setup();
        state.registry.create_session("delete-me".to_string());
        
        delete_session(State(state.clone()), Path("delete-me".to_string())).await;
        
        let Json(sessions) = list_sessions(State(state.clone())).await;
        assert_eq!(sessions.len(), 0);
    }
}