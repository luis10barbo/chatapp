pub mod session_db;
pub mod user;

use rusqlite::{Connection, Error};

use user::USER_TABLE_SQL;

const DB_NAME: &str = "database.sqlite";
pub fn get() -> Result<Database, rusqlite::Error> {
    let conn = Connection::open(DB_NAME).unwrap();
    let db = Database { conn };
    db.creation()?;
    Ok(db)
}

pub struct Database {
    conn: Connection,
}

impl Database {
    fn creation(&self) -> Result<(), Error> {
        self.conn.execute_batch(&format!(
            "BEGIN;
            {USER_TABLE_SQL}
            CREATE TABLE IF NOT EXISTS chats (
                chat_id INTEGER PRIMARY KEY AUTOINCREMENT,
                chat_name VARCHAR(32) NOT NULL DEFAULT \"\",
                chat_desc VARCHAR(512) DEFAULT \"\"
            );
            CREATE TABLE IF NOT EXISTS chat_messages (
                chat_message_id VARCHAR(26) PRIMARY KEY,
                chat_id INTEGER,
                user_id INTEGER,
                chat_message_message VARCHAR(512),
                date_created VARCHAR(32),

                FOREIGN KEY (chat_id) REFERENCES chats(chat_id),
                FOREIGN KEY (user_id) REFERENCES users(user_id)
            );
            COMMIT;"
        ))?;
        Ok(())
    }
}
