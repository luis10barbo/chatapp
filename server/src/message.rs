use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageTypes {
    JOIN,
    LEAVE,
    TEXT,
    ID,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SocketMessage {
    pub message_type: MessageTypes,
    pub message: String,
    pub id: Option<Uuid>,
}

impl SocketMessage {
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
                message_type: MessageTypes::TEXT,
                message: object.message,
                ..Default::default()
            });
        }
        None
    }
}

impl Default for SocketMessage {
    fn default() -> Self {
        Self {
            message_type: MessageTypes::TEXT,
            message: "".into(),
            id: None,
        }
    }
}
