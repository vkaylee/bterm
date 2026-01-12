use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite, Row};
use std::str::FromStr;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub role: String,
}

#[derive(Clone)]
pub struct Db {
    pub pool: Pool<Sqlite>,
}

impl Db {
    pub async fn new(database_url: &str) -> Result<Self> {
        let options = sqlx::sqlite::SqliteConnectOptions::from_str(database_url)?
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        let db = Self { pool };
        db.init().await?;
        Ok(db)
    }

    async fn init(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                role TEXT DEFAULT 'member'
            );
            "#
        )
        .execute(&self.pool)
        .await?;
        
        // Create default admin if not exists (for initial setup)
        // In a real app, we might use a CLI command, but here we auto-seed for convenience
        // We'll handle this logic in auth service or main if needed, but keeping DB clean is better.
        // For now, just the table.
        Ok(())
    }

    pub async fn create_user(&self, username: &str, password_hash: &str, role: &str) -> Result<User> {
        let id = sqlx::query(
            r#"
            INSERT INTO users (username, password_hash, role)
            VALUES (?, ?, ?)
            RETURNING id
            "#
        )
        .bind(username)
        .bind(password_hash)
        .bind(role)
        .fetch_one(&self.pool)
        .await?
        .get::<i64, _>("id");

        Ok(User {
            id,
            username: username.to_string(),
            password_hash: password_hash.to_string(),
            role: role.to_string(),
        })
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE username = ?"
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }
    
    pub async fn get_user_by_id(&self, id: i64) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }
}
