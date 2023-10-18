pub mod chat_db;
pub mod chat_message_db;
pub mod session_db;
pub mod user_db;

use rusqlite::{Connection, Error};

use user_db::USER_TABLE_SQL;

use self::chat_db::{CHAT_TABLE_SQL, CHAT_USERS_TABLE_SQL};
use self::chat_message_db::CHAT_MESSAGES_TABLE_SQL;

const DB_NAME: &str = "database.sqlite";
pub fn get() -> Result<Database, rusqlite::Error> {
    let conn = Connection::open(DB_NAME).unwrap();
    let db = Database { conn };
    db.creation()?;
    Ok(db)
}

#[derive(Debug)]
pub struct Database {
    conn: Connection,
}

impl Database {
    fn creation(&self) -> Result<(), Error> {
        self.conn.execute_batch(&format!(
            "BEGIN;
            {USER_TABLE_SQL}
            {CHAT_TABLE_SQL}
            {CHAT_MESSAGES_TABLE_SQL}
            {CHAT_USERS_TABLE_SQL}
            COMMIT;"
        ))?;
        Ok(())
    }
}
