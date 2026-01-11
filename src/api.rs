use axum::extract::{Json, State};
use axum::response::IntoResponse;
use std::sync::Arc;
use crate::session::{SessionRegistry, SessionInfo};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateSessionRequest {
    pub id: String,
}

pub async fn list_sessions(
    State(registry): State<Arc<SessionRegistry>>,
) -> Json<Vec<SessionInfo>> {
    println!("API: Listing sessions");
    Json(registry.list_sessions())
}

pub async fn create_session(
    State(registry): State<Arc<SessionRegistry>>,
    Json(payload): Json<CreateSessionRequest>,
) -> impl IntoResponse {
    println!("API: Creating session with ID: {}", payload.id);
    let _session = registry.create_session(payload.id);
    println!("API: Session created successfully.");
    Json("Created").into_response()
}

pub async fn delete_session(
    State(registry): State<Arc<SessionRegistry>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    // Sửa lại từ delete_session thành remove_session theo src/session.rs
    registry.remove_session(&id);
    axum::http::StatusCode::OK.into_response()
}