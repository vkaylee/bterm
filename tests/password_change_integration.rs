use bterminal::{create_app, session::SessionRegistry, db::Db, auth};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use std::sync::Arc;
use tokio::sync::broadcast;
use http_body_util::BodyExt;

#[tokio::test]
async fn test_forced_password_change_flow() {
    let (tx, _) = broadcast::channel(10);
    let registry = Arc::new(SessionRegistry::new(tx.clone()));
    let db = Db::new("sqlite::memory:").await.unwrap();
    
    // Create user (must_change_password defaults to true in Db::create_user now)
    let hash = auth::hash_password("initial_pass").unwrap();
    db.create_user("newuser", &hash, "member").await.unwrap();

    let app = create_app(tx, registry, db);

    // 1. Login
    let login_body = serde_json::to_string(&serde_json::json!({
        "username": "newuser",
        "password": "initial_pass"
    })).unwrap();

    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(login_body))
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    // Check response indicates must_change_password: true
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let user_resp: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(user_resp["must_change_password"], true);

    // Get cookie
    // We need to re-run login or get it from previous. Since into_body consumed it, we'll re-run or just store cookie.
    // Let's use a helper for login to get cookie and user info.
}

#[tokio::test]
async fn test_forbidden_until_password_changed() {
    let (tx, _) = broadcast::channel(10);
    let registry = Arc::new(SessionRegistry::new(tx.clone()));
    let db = Db::new("sqlite::memory:").await.unwrap();
    
    let hash = auth::hash_password("initial").unwrap();
    db.create_user("user1", &hash, "member").await.unwrap();

    let app = create_app(tx, registry, db);

    // Login
    let login_body = serde_json::to_string(&serde_json::json!({
        "username": "user1",
        "password": "initial"
    })).unwrap();
    let response = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/auth/login")
            .header("content-type", "application/json")
            .body(Body::from(login_body))
            .unwrap()
    ).await.unwrap();
    
    let cookie = response.headers().get("set-cookie").unwrap().to_str().unwrap().to_string();

    // Access protected resource -> Should be 403 (FORBIDDEN) because must_change_password is true
    let response = app.clone().oneshot(
        Request::builder()
            .uri("/api/sessions")
            .header("cookie", &cookie)
            .body(Body::empty())
            .unwrap()
    ).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    // Change password
    let change_body = serde_json::to_string(&serde_json::json!({
        "new_password": "new_secure_pass"
    })).unwrap();
    let response = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/auth/change-password")
            .header("cookie", &cookie)
            .header("content-type", "application/json")
            .body(Body::from(change_body))
            .unwrap()
    ).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Access protected resource again -> Should be 200 (OK) now
    let response = app.clone().oneshot(
        Request::builder()
            .uri("/api/sessions")
            .header("cookie", &cookie)
            .body(Body::empty())
            .unwrap()
    ).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
