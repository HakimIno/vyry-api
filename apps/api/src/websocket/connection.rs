use actix_ws::Session;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub type ConnectionId = Uuid;

#[derive(Clone)]
pub struct WsConnection {
    pub user_id: Uuid,
    pub device_id: i64,
    pub conn_id: ConnectionId,
    pub session: Session,
}

pub struct ConnectionManager {
    connections: Arc<RwLock<HashMap<ConnectionId, WsConnection>>>,
    user_connections: Arc<RwLock<HashMap<Uuid, Vec<ConnectionId>>>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            user_connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_connection(&self, conn: WsConnection) {
        let conn_id = conn.conn_id;
        let user_id = conn.user_id;

        self.connections.write().await.insert(conn_id, conn);

        self.user_connections
            .write()
            .await
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(conn_id);
    }

    pub async fn remove_connection(&self, conn_id: &ConnectionId) {
        if let Some(conn) = self.connections.write().await.remove(conn_id) {
            if let Some(conns) = self.user_connections.write().await.get_mut(&conn.user_id) {
                conns.retain(|id| id != conn_id);
            }
        }
    }

    #[allow(dead_code)]
    pub async fn get_user_connections(&self, user_id: &Uuid) -> Vec<WsConnection> {
        let user_conns = self.user_connections.read().await;
        let all_conns = self.connections.read().await;
        
        if let Some(conn_ids) = user_conns.get(user_id) {
            conn_ids.iter()
                .filter_map(|id| all_conns.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    #[allow(dead_code)]
    pub async fn get_device_connection(&self, user_id: &Uuid, device_id: i64) -> Option<WsConnection> {
        let connections = self.get_user_connections(user_id).await;
        connections.into_iter().find(|c| c.device_id == device_id)
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}
