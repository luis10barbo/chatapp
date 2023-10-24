use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    chat_message_db::{ChatMessage, ChatMessagesTable},
    Database,
};

pub const CHAT_TABLE_SQL: &str = "CREATE TABLE IF NOT EXISTS chats (
    chat_id VARCHAR(36) PRIMARY KEY,
    chat_name VARCHAR(32) NOT NULL DEFAULT \"\",
    chat_desc VARCHAR(512) DEFAULT \"\"
);";

pub const CHAT_USERS_TABLE_SQL: &str = "CREATE TABLE IF NOT EXISTS chat_users (
    chat_user_id INTEGER PRIMARY KEY AUTOINCREMENT,
    chat_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL
);";
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum ChatTypes {
    USER,
    GROUP,
}

#[derive(Debug, Serialize)]
pub struct Chat {
    chat_id: String,
    chat_name: String,
    chat_desc: String,
    chat_type: ChatTypes,
    last_message: Option<ChatMessage>,
}
pub trait ChatTable {
    fn create_chat(&self, nome: &str) -> Result<String, rusqlite::Error>;
    fn get_chats(&self) -> Result<Vec<Chat>, rusqlite::Error>;
    fn get_chat(&self, chat_id: &str, t: ChatTypes) -> Result<Chat, rusqlite::Error>;
}

impl ChatTable for Database {
    fn create_chat(&self, nome: &str) -> Result<String, rusqlite::Error> {
        let uuid = Uuid::new_v4();
        let mut stmt = self
            .conn
            .prepare("INSERT INTO chats (chat_id, chat_name) VALUES (?, ?)")?;
        stmt.execute(params![uuid.to_string(), nome])?;
        Ok(uuid.to_string())
    }
    fn get_chats(&self) -> Result<Vec<Chat>, rusqlite::Error> {
        let mut chats: Vec<Chat> = Vec::new();
        let mut stmt = self
            .conn
            .prepare("SELECT chat_id, chat_name, chat_desc FROM chats")?;
        let rows = stmt.query_map((), |row| {
            Ok(Chat {
                chat_id: row.get(0)?,
                chat_name: row.get(1)?,
                chat_desc: row.get(2)?,
                chat_type: ChatTypes::GROUP,
                last_message: None,
            })
        })?;

        for row in rows {
            let mut row = row?;
            let last_message = self.get_last_chat_message(row.chat_id.clone());
            let Ok(last_message) =  last_message else {
                println!("Error getting last message from chat {} {:?}", row.chat_id.clone(), last_message.unwrap_err());
                chats.push(row);
                continue;
            };
            row.last_message = Some(last_message);
            chats.push(row);
        }
        Ok(chats)
    }
    fn get_chat(&self, chat_id: &str, t: ChatTypes) -> Result<Chat, rusqlite::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT chat_id, chat_name, chat_desc FROM chats WHERE chat_id = ? LIMIT 1")?;
        let res = stmt.query_row(params![chat_id.to_string()], |row| {
            Ok(Chat {
                chat_id: row.get(0)?,
                chat_name: row.get(1)?,
                chat_desc: row.get(2)?,
                chat_type: ChatTypes::GROUP,
                last_message: None,
            })
        })?;
        Ok(res)
    }
}
