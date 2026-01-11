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
    match registry.create_session(payload.id) {
        Ok(_) => {
            println!("API: Session created successfully.");
            Json("Created").into_response()
        },
        Err(e) => {
            let error_msg = e.to_string();
            eprintln!("API: Error creating session: {}", error_msg);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, error_msg).into_response()
        }
    }
}

pub async fn delete_session(
    State(registry): State<Arc<SessionRegistry>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if registry.delete_session(&id) {
        axum::http::StatusCode::OK.into_response()
    } else {
        axum::http::StatusCode::NOT_FOUND.into_response()
    }
}
