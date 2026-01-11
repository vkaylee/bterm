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

    let env_port = std::env::var("PORT").ok().and_then(|p| p.parse::<u16>().ok());
    let mut ports = Vec::new();
    if let Some(p) = env_port {
        ports.push(p);
    }
    if !ports.contains(&3000) {
        ports.push(3000);
    }
    ports.push(0);

    let mut listener = None;
    for port in ports {
        let addr = format!("0.0.0.0:{port}");
        match tokio::net::TcpListener::bind(&addr).await {
            Ok(l) => {
                listener = Some(l);
                break;
            }
            Err(_) if port != 0 => continue,
            Err(e) => panic!("Failed to bind to any port: {e}"),
        }
    }

    let listener = listener.unwrap();
    let local_addr = listener.local_addr().unwrap();
    let port = local_addr.port();

    println!("🚀 BTerminal is running on http://localhost:{port}");
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
