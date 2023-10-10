// use actix_session::Session;
// use actix_web::HttpRequest;
// use serde::de::Error;

// use super::Database;

// pub const SESSION_TABLE_SQL: &str = "CREATE TABLE IF NOT EXISTS session (
//     session_id text PRIMARY KEY AUTOINCREMENT,
//     user_id INTEGER,

//     FOREIGN KEY (user_id) REFERENCES users(user_id)
// )";

// pub struct SessionStruct {
//     session_id: String,
//     user_id: usize,
// }

// pub trait SessionTable {
//     fn create_session(
//         &self,
//         request: HttpRequest,
//         session: Session,
//         user_id: usize,
//     ) -> Result<usize, rusqlite::Error>;
//     fn remove_session(&self, session_id: usize) -> Result<(), rusqlite::Error>;
//     fn read_session(&self, session_id: usize) -> Result<SessionStruct, rusqlite::Error>;
//     fn update_session(
//         &self,
//         session_id: usize,
//         user_id: usize,
//     ) -> Result<SessionStruct, rusqlite::Error>;
// }

// impl SessionTable for Database {
//     fn create_session(
//         &self,
//         request: HttpRequest,
//         session: Session,
//         user_id: usize,
//     ) -> Result<usize, rusqlite::Error> {
//         Ok(0)
//     }

//     fn remove_session(&self, session_id: usize) -> Result<(), rusqlite::Error> {
//         todo!()
//     }

//     fn read_session(&self, session_id: usize) -> Result<SessionStruct, rusqlite::Error> {
//         todo!()
//     }

//     fn update_session(
//         &self,
//         session_id: usize,
//         user_id: usize,
//     ) -> Result<SessionStruct, rusqlite::Error> {
//         todo!()
//     }
// }
