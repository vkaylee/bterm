#![deny(warnings)]
#![allow(unexpected_cfgs)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

pub mod pty_manager;
pub mod session;
pub mod ws;
pub mod api;

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

#[derive(RustEmbed)]
#[folder = "frontend/dist/"]
pub struct Assets;

pub fn create_app(registry: Arc<SessionRegistry>) -> Router {
    Router::new()
        .route("/api/sessions", get(api::list_sessions))
        .route("/api/sessions", post(api::create_session))
        .route("/api/sessions/{id}", axum::routing::delete(api::delete_session))
        .route("/ws/{session_id}", get(ws::ws_handler))
        .fallback(static_handler)
        .with_state(registry)
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

    #[tokio::test]
    async fn test_static_handler_index() {
        let registry = Arc::new(SessionRegistry::new());
        let app = create_app(registry);

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
        let registry = Arc::new(SessionRegistry::new());
        let app = create_app(registry);

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
        let registry = Arc::new(SessionRegistry::new());
        let app = create_app(registry);

        let response = app
            .oneshot(Request::builder().uri("/missing.css").body(axum::body::Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_static_handler_index_explicit() {
        let registry = Arc::new(SessionRegistry::new());
        let app = create_app(registry);

        let response = app
            .oneshot(Request::builder().uri("/index.html").body(axum::body::Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_static_handler_asset() {
        let registry = Arc::new(SessionRegistry::new());
        let app = create_app(registry);

        // Request an asset that exists in frontend/dist/assets/
        let response = app
            .oneshot(Request::builder().uri("/assets/xterm.css").body(axum::body::Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get(header::CONTENT_TYPE).unwrap(), "text/css");
    }
}
