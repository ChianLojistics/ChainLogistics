use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageType {
    #[serde(rename = "subscribe")]
    Subscribe { channel: String },
    #[serde(rename = "unsubscribe")]
    Unsubscribe { channel: String },
    #[serde(rename = "event")]
    Event { channel: String, data: serde_json::Value },
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "pong")]
    Pong,
    #[serde(rename = "error")]
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub id: String,
    pub timestamp: i64,
    #[serde(flatten)]
    pub message_type: MessageType,
}

impl WebSocketMessage {
    pub fn new(message_type: MessageType) -> Self {
        WebSocketMessage {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            message_type,
        }
    }

    pub fn subscribe(channel: String) -> Self {
        Self::new(MessageType::Subscribe { channel })
    }

    pub fn unsubscribe(channel: String) -> Self {
        Self::new(MessageType::Unsubscribe { channel })
    }

    pub fn event(channel: String, data: serde_json::Value) -> Self {
        Self::new(MessageType::Event { channel, data })
    }

    pub fn ping() -> Self {
        Self::new(MessageType::Ping)
    }

    pub fn pong() -> Self {
        Self::new(MessageType::Pong)
    }

    pub fn error(message: String) -> Self {
        Self::new(MessageType::Error { message })
    }
}
