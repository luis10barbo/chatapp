use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use rusqlite::params;
use serde::{Deserialize, Serialize};

use super::Database;

pub const USER_TABLE_SQL: &str = "CREATE TABLE IF NOT EXISTS users (
    user_id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_nick VARCHAR(32) UNIQUE NOT NULL,
    password_hash VARCHAR(64) NOT NULL,
    password_salt VARCHAR(32) NOT NULL,
    user_name VARCHAR(32),
    user_status VARCHAR(64) DEFAULT \"\",
    user_email VARCHAR(64),
    user_image TEXT
    
);";

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub user_id: i64,
    pub user_nick: String,
    pub user_name: Option<String>,
    pub user_status: Option<String>,
    pub user_email: Option<String>,
    pub user_image: Option<String>,
}

pub trait UserTable {
    fn create_user(&self, nickname: String, password: String) -> Result<i64, rusqlite::Error>;

    fn login_user(
        &self,
        nickname: String,
        password: String,
    ) -> Result<Option<i64>, rusqlite::Error>;

    fn get_user(&self, id: i64) -> Result<User, rusqlite::Error>;
    fn update_user(&self, user: User) -> Result<usize, rusqlite::Error>;
}

impl UserTable for Database {
    fn create_user(&self, nickname: String, password: String) -> Result<i64, rusqlite::Error> {
        let argon = Argon2::default();

        let salt = SaltString::generate(&mut OsRng);
        let hash = argon
            .hash_password(password.as_bytes(), &salt)
            .unwrap()
            .to_string();
        let mut stmt = self.conn.prepare(
            "INSERT INTO users (user_nick, password_hash, password_salt) VALUES (?, ?, ?)",
        )?;
        stmt.execute(params![nickname, hash, salt.to_string()])?;
        Ok(self.conn.last_insert_rowid())
    }

    fn login_user(
        &self,
        nickname: String,
        password: String,
    ) -> Result<Option<i64>, rusqlite::Error> {
        struct PasswordSelection {
            id: i64,
            password_hash: String,
        }

        let mut stmt = self
            .conn
            .prepare("SELECT user_id, password_hash FROM users WHERE user_nick = ?")?;
        let password_query = stmt.query_row(params![nickname], |row| {
            Ok(PasswordSelection {
                id: row.get(0)?,
                password_hash: row.get(1)?,
            })
        })?;

        let parsed_hash = PasswordHash::new(&password_query.password_hash).unwrap();
        let res = Argon2::default().verify_password(password.as_bytes(), &parsed_hash);
        if res.is_err() {
            return Ok(None);
        }

        Ok(Some(password_query.id))
    }

    fn get_user(&self, id: i64) -> Result<User, rusqlite::Error> {
        let mut stmt = self.conn.prepare("SELECT user_id, user_nick, user_name, user_status, user_email, user_image FROM users WHERE user_id = ? LIMIT 1")?;
        let user: User = stmt.query_row(params![id], |row| {
            Ok(User {
                user_id: row.get(0)?,
                user_nick: row.get(1)?,
                user_name: row.get(2)?,
                user_status: row.get(3)?,
                user_email: row.get(4)?,
                user_image: row.get(5)?,
            })
        })?;
        Ok(user)
    }
    fn update_user(&self, user: User) -> Result<usize, rusqlite::Error> {
        let mut stmt = self.conn.prepare("UPDATE users SET user_nick=?, user_name=?, user_status=?, user_email=?, user_image=? WHERE user_id=?")?;
        stmt.execute(params![
            user.user_nick,
            user.user_name,
            user.user_status,
            user.user_email,
            user.user_image,
            user.user_id
        ])
    }
}
