use std::sync::Arc;
use bterminal::{create_app, session::SessionRegistry, db::Db};

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() {
    let (tx, _rx) = tokio::sync::broadcast::channel(100);
    let registry = Arc::new(SessionRegistry::new(tx.clone()));
    
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:bterminal.db".to_string());
    println!("Database URL: {}", db_url);
    
    let db = Db::new(&db_url).await.expect("Failed to initialize database");

    // Auto-create admin user for MVP (admin / admin)
    // In production this should be done via a setup script or CLI
    let admin_username = "admin";
    if db.get_user_by_username(admin_username).await.unwrap().is_none() {
        println!("Creating default admin user...");
        let password_hash = bterminal::auth::hash_password("admin").expect("Failed to hash password");
        match db.create_user(admin_username, &password_hash, "admin").await {
            Ok(_) => println!("Default admin user created (admin/admin)"),
            Err(e) => println!("Failed to create admin user: {}", e),
        }
    }

    let app = create_app(tx, registry, db);

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