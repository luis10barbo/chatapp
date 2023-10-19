use rusqlite::params;
use serde::Serialize;
use uuid::Uuid;

use super::Database;

pub const CHAT_MESSAGES_TABLE_SQL: &str = "CREATE TABLE IF NOT EXISTS chat_messages (
    chat_message_id VARCHAR(36) PRIMARY KEY,
    chat_id VARCHAR(36) NOT NULL,
    user_id INTEGER NOT NULL,
    message VARCHAR(512),
    date_created VARCHAR(32),

    FOREIGN KEY (chat_id) REFERENCES chats(chat_id),
    FOREIGN KEY (user_id) REFERENCES users(user_id)
);";

#[derive(Debug, Serialize)]
pub struct ChatMessage {
    pub id: String,
    pub message: String,
    pub date_created: String,
    pub user_id: i64,
}

pub trait ChatMessagesTable {
    fn insert_message(&self, chat_message: InsertChatMessage) -> Result<String, rusqlite::Error>;
    fn get_chat_messages(
        &self,
        chat_id: String,
        offset: usize,
    ) -> Result<Vec<ChatMessage>, rusqlite::Error>;
}

pub struct InsertChatMessage<'t> {
    pub chat_id: String,
    pub user_id: i64,
    pub message: &'t str,
    pub date_created: String,
}
impl ChatMessagesTable for Database {
    fn insert_message(&self, chat_message: InsertChatMessage) -> Result<String, rusqlite::Error> {
        let message_id = Uuid::new_v4().to_string();
        let res = self.conn.execute(
            "INSERT INTO chat_messages (chat_message_id, chat_id, user_id, message, date_created) VALUES (?, ?, ?, ?, ?)",
            params![
                message_id,
                chat_message.chat_id,
                chat_message.user_id,
                chat_message.message,
                chat_message.date_created
            ],
        );
        if let Err(err) = res {
            return Err(err);
        }
        Ok(message_id)
    }

    fn get_chat_messages(
        &self,
        chat_id: String,
        offset: usize,
    ) -> Result<Vec<ChatMessage>, rusqlite::Error> {
        let mut messages = Vec::new();
        let mut stmt = self
            .conn
            .prepare("SELECT chat_message_id, user_id, message, date_created FROM chat_messages WHERE chat_id = ? ORDER BY datetime(date_created) DESC LIMIT 10 OFFSET ?")?;

        let query = stmt.query_map(params![chat_id, offset], |row| {
            Ok(ChatMessage {
                id: row.get(0)?,
                user_id: row.get(1)?,
                message: row.get(2)?,
                date_created: row.get(3)?,
            })
        })?;

        for message in query {
            messages.push(message?);
        }
        messages.reverse();
        Ok(messages)
    }
}
