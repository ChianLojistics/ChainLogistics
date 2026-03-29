use crate::websocket::{ConnectionManager, WebSocketMessage, MessageType};
use tokio::sync::mpsc;
use futures::{SinkExt, StreamExt};
use warp::ws::{WebSocket, Message};

pub struct WebSocketHandler {
    manager: ConnectionManager,
}

impl WebSocketHandler {
    pub fn new(manager: ConnectionManager) -> Self {
        WebSocketHandler { manager }
    }

    pub async fn handle_connection(&self, ws: WebSocket) {
        let (mut user_ws_tx, mut user_ws_rx) = ws.split();
        let (tx, mut rx) = mpsc::unbounded_channel();

        let connection_id = self.manager.add_connection(tx).await;
        let manager = self.manager.clone();
        let conn_id = connection_id.clone();

        // Spawn task to handle incoming messages
        tokio::spawn(async move {
            while let Some(result) = user_ws_rx.next().await {
                match result {
                    Ok(msg) => {
                        if let Ok(text) = msg.to_str() {
                            if let Ok(ws_msg) = serde_json::from_str::<WebSocketMessage>(text) {
                                match ws_msg.message_type {
                                    MessageType::Subscribe { channel } => {
                                        let _ = manager.subscribe(&conn_id, &channel).await;
                                    }
                                    MessageType::Unsubscribe { channel } => {
                                        let _ = manager.unsubscribe(&conn_id, &channel).await;
                                    }
                                    MessageType::Ping => {
                                        let pong = WebSocketMessage::pong();
                                        let _ = manager
                                            .send_to_connection(&conn_id, serde_json::to_string(&pong).unwrap())
                                            .await;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
            manager.remove_connection(&conn_id).await;
        });

        // Spawn task to handle outgoing messages
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if user_ws_tx.send(Message::text(msg)).await.is_err() {
                    break;
                }
            }
        });
    }
}
