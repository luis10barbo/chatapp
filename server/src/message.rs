use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageType {
    JOIN,
    LEAVE,
    TEXT,
    ID,
    INIT,
    DISCONNECTED,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SocketMessage {
    pub message_type: MessageType,
    pub message: String,
    pub id: Option<i64>,
    pub date: String,
}

impl SocketMessage {
    pub fn new(message: String, message_type: MessageType, id: Option<i64>) -> Self {
        Self {
            message,
            message_type,
            id,
            ..Default::default()
        }
    }
    pub fn parse(message: &str) -> Self {
        let res = Self::parse_failable(message);
        if res.is_none() {
            return Self::default();
        }
        res.unwrap()
    }
    pub fn parse_failable(message: &str) -> Option<Self> {
        let res = serde_json::from_str(message);
        if res.is_ok() {
            let object: SocketMessage = res.unwrap();
            return Some(Self {
                message_type: MessageType::TEXT,
                message: object.message,
                ..Default::default()
            });
        }
        None
    }
}

const DATE_FORMATTING: &str = "%Y-%m-%d %H:%M:%S";

fn format_date(date_time: DateTime<Utc>) -> String {
    format!("{}", date_time.format(DATE_FORMATTING))
}

impl Default for SocketMessage {
    fn default() -> Self {
        Self {
            message_type: MessageType::TEXT,
            message: "".into(),
            id: None,
            date: format_date(Utc::now()),
        }
    }
}
