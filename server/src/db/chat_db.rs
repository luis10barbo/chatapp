use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::message::format_date;

use super::{
    chat_message_db::{ChatMessage, ChatMessagesTable},
    Database,
};

pub const CHAT_TABLE_SQL: &str = "CREATE TABLE IF NOT EXISTS chats (
    chat_id VARCHAR(36) PRIMARY KEY,
    user_id INTEGER,
    chat_name VARCHAR(32) NOT NULL DEFAULT \"\",
    chat_desc VARCHAR(512) DEFAULT \"\",
    date_created VARCHAR(32),
    
    FOREIGN KEY (user_id) REFERENCES users(user_id)
);";

pub const CHAT_USERS_TABLE_SQL: &str = "CREATE TABLE IF NOT EXISTS chat_users (
    chat_user_id INTEGER PRIMARY KEY AUTOINCREMENT,
    chat_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    FOREIGN KEY (chat_id) REFERENCES chats(chat_id),
    FOREIGN KEY (user_id) REFERENCES users(user_id)
);";
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum ChatTypes {
    USER,
    GROUP,
}

#[derive(Debug, Serialize)]
pub struct Chat {
    chat_id: String,
    creator_id: i64,
    chat_name: String,
    chat_desc: String,
    date_created: String,
    chat_type: ChatTypes,
    last_message: Option<ChatMessage>,
}
pub trait ChatTable {
    fn create_chat(&self, nome: &str, id_usuario: i64) -> Result<String, rusqlite::Error>;
    fn get_chats(&self) -> Result<Vec<Chat>, rusqlite::Error>;
    fn get_chat(&self, chat_id: &str, t: ChatTypes) -> Result<Chat, rusqlite::Error>;
    fn remove_chat(&self, chat_id: &str) -> Result<usize, rusqlite::Error>;
}

impl ChatTable for Database {
    fn create_chat(&self, nome: &str, id_usuario: i64) -> Result<String, rusqlite::Error> {
        let uuid = Uuid::new_v4();
        let mut stmt = self.conn.prepare(
            "INSERT INTO chats (chat_id, chat_name, user_id, date_created) VALUES (?, ?, ?, ?)",
        )?;
        stmt.execute(params![
            uuid.to_string(),
            nome,
            id_usuario,
            format_date(Utc::now())
        ])?;
        Ok(uuid.to_string())
    }
    fn get_chats(&self) -> Result<Vec<Chat>, rusqlite::Error> {
        let mut chats: Vec<Chat> = Vec::new();
        let mut stmt = self
            .conn
            .prepare("SELECT chat_id, chat_name, chat_desc, user_id, date_created FROM chats")?;
        let rows = stmt.query_map((), |row| {
            Ok(Chat {
                chat_id: row.get(0)?,
                chat_name: row.get(1)?,
                chat_desc: row.get(2)?,
                creator_id: row.get(3)?,
                date_created: row.get(4)?,
                chat_type: ChatTypes::GROUP,
                last_message: None,
            })
        })?;

        for row in rows {
            let mut row = row?;
            let last_message = self.get_last_chat_message(row.chat_id.clone());
            if let Ok(last_message) = last_message {
                row.last_message = Some(last_message);
            } else {
                let err = last_message.unwrap_err();
                if err != rusqlite::Error::QueryReturnedNoRows {
                    println!(
                        "Unknown error getting last message from chat {} {:?}",
                        row.chat_id.clone(),
                        err.sqlite_error_code()
                    );
                }
            };
            chats.push(row);
        }
        Ok(chats)
    }
    fn get_chat(&self, chat_id: &str, _: ChatTypes) -> Result<Chat, rusqlite::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT chat_id, chat_name, chat_desc, user_id, date_created FROM chats WHERE chat_id = ? LIMIT 1")?;
        let res = stmt.query_row(params![chat_id.to_string()], |row| {
            Ok(Chat {
                chat_id: row.get(0)?,
                chat_name: row.get(1)?,
                chat_desc: row.get(2)?,
                creator_id: row.get(3)?,
                date_created: row.get(4)?,
                chat_type: ChatTypes::GROUP,
                last_message: None,
            })
        })?;
        Ok(res)
    }

    fn remove_chat(&self, chat_id: &str) -> Result<usize, rusqlite::Error> {
        // let res = self.conn.prepare("DELETE FROM ")
        let res = self
            .conn
            .prepare("DELETE FROM chats WHERE chat_id = ?")?
            .execute(params![chat_id])?;
        log::debug!("{:?}", res);
        Ok(res)
    }
}
