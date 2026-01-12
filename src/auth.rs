use axum::{
    extract::{State, Request},
    middleware::Next,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;
use std::sync::Arc;
use crate::{AppState, db::User};

pub const SESSION_USER_KEY: &str = "user_id";

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    id: i64,
    username: String,
    role: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            role: user.role,
        }
    }
}

pub fn hash_password(password: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!(e))?
        .to_string();
    Ok(password_hash)
}

pub fn verify_password(password: &str, password_hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(password_hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    session: Session,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let user = match state.db.get_user_by_username(&payload.username).await {
        Ok(Some(u)) => u,
        Ok(None) => return (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
    };

    if !verify_password(&payload.password, &user.password_hash) {
        return (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response();
    }

    if let Err(_) = session.insert(SESSION_USER_KEY, user.id).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create session").into_response();
    }

    (StatusCode::OK, Json(UserResponse::from(user))).into_response()
}

pub async fn logout(session: Session) -> impl IntoResponse {
    let _ = session.flush().await;
    (StatusCode::OK, "Logged out").into_response()
}

pub async fn me(
    State(state): State<Arc<AppState>>,
    session: Session,
) -> impl IntoResponse {
    let user_id: Option<i64> = match session.get(SESSION_USER_KEY).await {
        Ok(id) => id,
        Err(_) => return (StatusCode::UNAUTHORIZED, "Session error").into_response(),
    };

    let user_id = match user_id {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, "Not authenticated").into_response(),
    };

    match state.db.get_user_by_id(user_id).await {
        Ok(Some(user)) => (StatusCode::OK, Json(UserResponse::from(user))).into_response(),
        Ok(None) => {
            let _ = session.flush().await; // User deleted?
            (StatusCode::UNAUTHORIZED, "User not found").into_response()
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
    }
}

pub async fn require_auth(
    session: Session,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user_id: Option<i64> = session.get(SESSION_USER_KEY).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match user_id {
        Some(_) => Ok(next.run(request).await),
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(me))
}