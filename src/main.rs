use std::sync::Arc;
use bterminal::{create_app, session::SessionRegistry};

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() {
    let registry = Arc::new(SessionRegistry::new());
    let app = create_app(registry);

    let listener = bind_listener().await;
    let local_addr = listener.local_addr().unwrap();
    let port = local_addr.port();

    println!("🚀 BTerminal is running on http://localhost:{port}");
    println!("Press Ctrl+C to stop the server");
    
    axum::serve(listener, app).await.unwrap();
}

#[cfg(not(tarpaulin_include))]
async fn bind_listener() -> tokio::net::TcpListener {
    let env_port = std::env::var("PORT").ok().and_then(|p| p.parse::<u16>().ok());
    let mut ports = Vec::new();
    if let Some(p) = env_port {
        ports.push(p);
    }
    if !ports.contains(&3000) {
        ports.push(3000);
    }
    ports.push(0);

    for port in ports {
        let addr = format!("0.0.0.0:{port}");
        match tokio::net::TcpListener::bind(&addr).await {
            Ok(l) => return l,
            Err(_) if port != 0 => continue,
            Err(e) => panic!("Failed to bind to any port: {e}"),
        }
    }
    unreachable!()
}
