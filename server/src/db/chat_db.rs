use rusqlite::params;
use serde::Serialize;
use uuid::Uuid;

use super::Database;

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
#[derive(Debug, Serialize)]
pub struct Chat {
    chat_id: String,
    chat_name: String,
    chat_desc: String,
}
pub trait ChatTable {
    fn create_chat(&self, nome: &str) -> Result<Uuid, rusqlite::Error>;
    // fn get_chat_messages(
    //     &self,
    //     id: i64,
    //     offset: usize,
    // ) -> Result<Vec<ChatMessage>, rusqlite::Error>;
    fn send_chat_message(&self, msg: ChatMessage) -> Result<String, rusqlite::Error>;
    fn get_chats(&self) -> Result<Vec<Chat>, rusqlite::Error>;
    fn get_chat(&self, chat_id: Uuid) -> Result<Chat, rusqlite::Error>;
}

pub struct ChatMessage {
    pub chat_id: String,
    pub date_created: String,
    pub message: String,
    pub message_id: String,
    pub user_id: i32,
}

impl ChatTable for Database {
    fn create_chat(&self, nome: &str) -> Result<Uuid, rusqlite::Error> {
        let uuid = Uuid::new_v4();
        let mut stmt = self
            .conn
            .prepare("INSERT INTO chats (chat_id, chat_name) VALUES (?, ?)")?;
        stmt.execute(params![uuid.to_string(), nome])?;
        Ok(uuid)
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
            })
        })?;
        for row in rows {
            chats.push(row?);
        }
        Ok(chats)
    }
    fn get_chat(&self, chat_id: Uuid) -> Result<Chat, rusqlite::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT chat_id, chat_name, chat_desc FROM chats WHERE chat_id = ? LIMIT 1")?;
        let res = stmt.query_row(params![chat_id.to_string()], |row| {
            Ok(Chat {
                chat_id: row.get(0)?,
                chat_name: row.get(1)?,
                chat_desc: row.get(2)?,
            })
        })?;
        Ok(res)
    }
    fn send_chat_message(&self, msg: ChatMessage) -> Result<String, rusqlite::Error> {
        let mut stmt = self.conn.prepare("INSERT INTO chat_messages (chat_message_id, chat_id, user_id, message, date_created) VALUES (?, ?, ?, ?, ?)")?;
        stmt.execute(params![
            msg.message_id.to_string(),
            msg.chat_id.to_string(),
            msg.user_id,
            msg.message,
            msg.date_created
        ])?;
        Ok(msg.message_id)
    }
    // fn get_chat_messages(
    //     &self,
    //     chat_id: String,
    //     offset: usize,
    // ) -> Result<Vec<ChatMessage>, rusqlite::Error> {
    //     let limit = 10;
    //     let mut stmt = self.conn.prepare(&format!(
    //         "SELECT c.chat_id, cm.date_created, cm.message, cm.chat_message_id, cm.user_id FROM chats c
    //     INNER JOIN chat_messages cm ON c.chat_id = cm.chat_id
    //     WHERE c.chat_id = ? ORDER BY datetime(cm.date_created) DESC LIMIT {limit} OFFSET ?"),
    //     )?;
    //     let res = stmt.query_map(params![chat_id, offset * limit], |row| {
    //         Ok(ChatMessage {
    //             chat_id: row.get(0)?,
    //             date_created: row.get(1)?,
    //             message: row.get(2)?,
    //             message_id: row.get(3)?,
    //             user_id: row.get(4)?,
    //         })
    //     })?;
    //     let mut chat_messages: Vec<ChatMessage> = Vec::new();
    //     for chat_message in res {
    //         chat_messages.push(chat_message?);
    //     }
    //     Ok(chat_messages)
    // }
}
