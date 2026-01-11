#![deny(warnings)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

mod pty_manager;
mod session;
mod ws;
mod api;

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
struct Assets;

#[tokio::main]
async fn main() {
    let registry = Arc::new(SessionRegistry::new());

    let app = Router::new()
        .route("/api/sessions", get(api::list_sessions))
        .route("/api/sessions", post(api::create_session))
        .route("/api/sessions/{id}", axum::routing::delete(api::delete_session))
        .route("/ws/{session_id}", get(ws::ws_handler))
        .fallback(static_handler)
        .with_state(registry)
        .layer(CorsLayer::permissive());

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("🚀 BTerminal is running on http://{addr}");
    println!("Press Ctrl+C to stop the server");
    
    axum::serve(listener, app).await.unwrap();
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
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

fn index_html() -> Response {

    match Assets::get("index.html") {

        Some(content) => Html(content.data).into_response(),

        None => StatusCode::NOT_FOUND.into_response(),

    }

}
