use bterminal::{create_app, session::SessionRegistry, db::Db, auth};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use std::sync::Arc;
use tokio::sync::broadcast;

#[tokio::test]
async fn test_auth_flow() {
    let (tx, _) = broadcast::channel(10);
    let registry = Arc::new(SessionRegistry::new(tx.clone()));
    let db = Db::new("sqlite::memory:").await.unwrap();
    
    // Create user
    let hash = auth::hash_password("password123").unwrap();
    db.create_user("testuser", &hash, "member").await.unwrap();

    let app = create_app(tx, registry, db);

    // 1. Fail without auth
    let response = app.clone()
        .oneshot(Request::builder().uri("/api/sessions").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // 2. Login
    let login_body = serde_json::to_string(&serde_json::json!({
        "username": "testuser",
        "password": "password123"
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
    
    // Get cookie
    let headers = response.headers();
    let cookie = headers.get("set-cookie").expect("No cookie returned").to_str().unwrap().to_string();

    // 2.5 Change password (mandatory for new users)
    let change_body = serde_json::to_string(&serde_json::json!({
        "new_password": "newpassword123"
    })).unwrap();

    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/change-password")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(change_body))
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 3. Success with auth
    let response = app.clone()
        .oneshot(
            Request::builder()
                .uri("/api/sessions")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
