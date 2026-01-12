#![deny(warnings)]
#![allow(unexpected_cfgs)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::must_use_candidate)]

pub mod pty_manager;
pub mod session;
pub mod ws;
pub mod api;
pub mod db;
pub mod auth;

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use crate::session::SessionRegistry;
use tower_http::cors::CorsLayer;
use rust_embed::RustEmbed;
use axum::response::{Html, IntoResponse, Response};
use axum::http::{header, StatusCode, Uri};
use tokio::sync::broadcast;
use serde::Serialize;
use tower_sessions::{SessionManagerLayer, MemoryStore, Expiry};
use time::Duration;

#[derive(RustEmbed)]
#[folder = "frontend/dist/"]
pub struct Assets;

#[derive(Clone, Serialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum GlobalEvent {
    SessionCreated(String),
    SessionDeleted(String),
}

pub struct AppState {
    pub registry: Arc<SessionRegistry>,
    pub tx: broadcast::Sender<GlobalEvent>,
    pub db: db::Db,
}

pub fn create_app(tx: broadcast::Sender<GlobalEvent>, registry: Arc<SessionRegistry>, db: db::Db) -> Router {
    let state = Arc::new(AppState { registry, tx, db });

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::hours(24)));

    let protected_routes = Router::new()
        .nest("/api", Router::new()
            .route("/sessions", get(api::list_sessions))
            .route("/sessions", post(api::create_session))
            .route("/sessions/{id}", axum::routing::delete(api::delete_session))
            .route("/events", get(api::events_handler))
        )
        .route("/ws/{session_id}", get(ws::ws_handler))
        .layer(axum::middleware::from_fn(auth::require_auth));

    let auth_routes = auth::routes();

    Router::new()
        .merge(protected_routes)
        .nest("/api/auth", auth_routes)
        .layer(session_layer)
        .fallback(static_handler)
        .with_state(state)
        .layer(CorsLayer::permissive())
}

pub async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    if path.is_empty() || path == "index.html" {
        return index_html();
    }

    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(axum::body::Body::from(content.data))
                .unwrap()
        }
        None => {
            if path.contains('.') {
                StatusCode::NOT_FOUND.into_response()
            } else {
                index_html()
            }
        }
    }
}

pub fn index_html() -> Response {
    match Assets::get("index.html") {
        Some(content) => Html(content.data).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use tower::ServiceExt;
    use axum::http::Request;

    async fn setup_app() -> Router {
        let (tx, _) = broadcast::channel(10);
        let registry = Arc::new(SessionRegistry::new(tx.clone()));
        // Setup in-memory DB for tests
        let db = db::Db::new("sqlite::memory:").await.unwrap();
        create_app(tx, registry, db)
    }

    #[tokio::test]
    async fn test_static_handler_index() {
        let app = setup_app().await;

        let response = app
            .oneshot(Request::builder().uri("/").body(axum::body::Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        assert!(body.starts_with(b"<!DOCTYPE html>"));
    }

    #[tokio::test]
    async fn test_static_handler_missing_file_falls_back_to_index() {
        let app = setup_app().await;

        let response = app
            .oneshot(Request::builder().uri("/random-path").body(axum::body::Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        assert!(body.starts_with(b"<!DOCTYPE html>"));
    }

    #[tokio::test]
    async fn test_static_handler_missing_file_with_extension_returns_404() {
        let app = setup_app().await;

        let response = app
            .oneshot(Request::builder().uri("/missing.css").body(axum::body::Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
