use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

pub type ConnectionId = String;
pub type Channel = String;
pub type Sender = mpsc::UnboundedSender<String>;

#[derive(Clone)]
pub struct ConnectionManager {
    connections: Arc<RwLock<HashMap<ConnectionId, Connection>>>,
    channels: Arc<RwLock<HashMap<Channel, HashSet<ConnectionId>>>>,
}

#[derive(Clone)]
struct Connection {
    id: ConnectionId,
    sender: Sender,
    subscribed_channels: HashSet<Channel>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        ConnectionManager {
            connections: Arc::new(RwLock::new(HashMap::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_connection(&self, sender: Sender) -> ConnectionId {
        let id = Uuid::new_v4().to_string();
        let connection = Connection {
            id: id.clone(),
            sender,
            subscribed_channels: HashSet::new(),
        };

        let mut connections = self.connections.write().await;
        connections.insert(id.clone(), connection);
        id
    }

    pub async fn remove_connection(&self, connection_id: &ConnectionId) {
        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.remove(connection_id) {
            let mut channels = self.channels.write().await;
            for channel in conn.subscribed_channels {
                if let Some(subs) = channels.get_mut(&channel) {
                    subs.remove(connection_id);
                }
            }
        }
    }

    pub async fn subscribe(&self, connection_id: &ConnectionId, channel: &Channel) -> Result<(), String> {
        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.get_mut(connection_id) {
            conn.subscribed_channels.insert(channel.clone());
        } else {
            return Err("Connection not found".to_string());
        }

        let mut channels = self.channels.write().await;
        channels
            .entry(channel.clone())
            .or_insert_with(HashSet::new)
            .insert(connection_id.clone());

        Ok(())
    }

    pub async fn unsubscribe(&self, connection_id: &ConnectionId, channel: &Channel) -> Result<(), String> {
        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.get_mut(connection_id) {
            conn.subscribed_channels.remove(channel);
        }

        let mut channels = self.channels.write().await;
        if let Some(subs) = channels.get_mut(channel) {
            subs.remove(connection_id);
        }

        Ok(())
    }

    pub async fn broadcast(&self, channel: &Channel, message: String) {
        let channels = self.channels.read().await;
        if let Some(subscribers) = channels.get(channel) {
            let connections = self.connections.read().await;
            for connection_id in subscribers {
                if let Some(conn) = connections.get(connection_id) {
                    let _ = conn.sender.send(message.clone());
                }
            }
        }
    }

    pub async fn send_to_connection(&self, connection_id: &ConnectionId, message: String) -> Result<(), String> {
        let connections = self.connections.read().await;
        if let Some(conn) = connections.get(connection_id) {
            conn.sender.send(message).map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("Connection not found".to_string())
        }
    }

    pub async fn get_channel_subscribers(&self, channel: &Channel) -> usize {
        let channels = self.channels.read().await;
        channels.get(channel).map(|s| s.len()).unwrap_or(0)
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}
