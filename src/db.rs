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
    pub must_change_password: bool,
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
                role TEXT DEFAULT 'member',
                must_change_password BOOLEAN DEFAULT 1
            );
            "#
        )
        .execute(&self.pool)
        .await?;
        
        // Cập nhật schema nếu cột chưa tồn tại (cho các DB cũ)
        let has_column = sqlx::query("PRAGMA table_info(users)")
            .fetch_all(&self.pool)
            .await?
            .iter()
            .any(|row| row.get::<String, _>("name") == "must_change_password");

        if !has_column {
            sqlx::query("ALTER TABLE users ADD COLUMN must_change_password BOOLEAN DEFAULT 1")
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }

    pub async fn create_user(&self, username: &str, password_hash: &str, role: &str) -> Result<User> {
        self.create_user_with_pwd_policy(username, password_hash, role, true).await
    }

    pub async fn create_user_with_pwd_policy(&self, username: &str, password_hash: &str, role: &str, must_change: bool) -> Result<User> {
        let row = sqlx::query(
            r#"
            INSERT INTO users (username, password_hash, role, must_change_password)
            VALUES (?, ?, ?, ?)
            RETURNING id, must_change_password
            "#
        )
        .bind(username)
        .bind(password_hash)
        .bind(role)
        .bind(must_change)
        .fetch_one(&self.pool)
        .await?;
        
        let id = row.get::<i64, _>("id");
        let must_change_password = row.get::<bool, _>("must_change_password");

        Ok(User {
            id,
            username: username.to_string(),
            password_hash: password_hash.to_string(),
            role: role.to_string(),
            must_change_password,
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

    pub async fn update_password(&self, user_id: i64, new_hash: &str) -> Result<()> {
        sqlx::query(
            "UPDATE users SET password_hash = ?, must_change_password = 0 WHERE id = ?"
        )
        .bind(new_hash)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
