use crate::types::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};

type ClientSender = mpsc::UnboundedSender<axum::extract::ws::Message>;

pub struct Hub {
    desktop: Arc<RwLock<Option<ClientSender>>>,
    mobiles: Arc<RwLock<HashMap<String, ClientSender>>>,
    device_sessions: Arc<Mutex<HashMap<String, DeviceSession>>>,
}

impl Hub {
    pub fn new() -> Self {
        Self {
            desktop: Arc::new(RwLock::new(None)),
            mobiles: Arc::new(RwLock::new(HashMap::new())),
            device_sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn register_desktop(&self, sender: ClientSender) {
        let mut desktop = self.desktop.write().await;
        *desktop = Some(sender);
    }

    pub async fn close_old_desktop(&self) {
        let mut desktop = self.desktop.write().await;
        if let Some(old_sender) = desktop.take() {
            let _ = old_sender.send(axum::extract::ws::Message::Close(None));
        }
    }

    pub async fn register_mobile(&self, id: String, sender: ClientSender) {
        let mut mobiles = self.mobiles.write().await;
        mobiles.insert(id.clone(), sender);
    }

    pub async fn unregister_desktop(&self) {
        let mut desktop = self.desktop.write().await;
        *desktop = None;
    }

    pub async fn unregister_mobile(&self, id: &str) {
        let mut mobiles = self.mobiles.write().await;
        mobiles.remove(id);
    }

    pub async fn send_to_desktop(&self, msg: WSMessage) -> Result<(), Box<dyn std::error::Error>> {
        let desktop = self.desktop.read().await;
        if let Some(sender) = desktop.as_ref() {
            let json = serde_json::to_string(&msg)?;
            sender.send(axum::extract::ws::Message::Text(json))?;
        }
        Ok(())
    }

    pub async fn broadcast_to_mobiles(&self, msg: WSMessage) {
        let mobiles = self.mobiles.read().await;
        let json = match serde_json::to_string(&msg) {
            Ok(j) => j,
            Err(_) => return,
        };

        for sender in mobiles.values() {
            let _ = sender.send(axum::extract::ws::Message::Text(json.clone()));
        }
    }

    pub async fn update_device_session(&self, device_id: String, device_name: String, content: String) {
        let mut sessions = self.device_sessions.lock().await;
        let session = sessions.entry(device_id.clone()).or_insert(DeviceSession {
            device_id: device_id.clone(),
            device_name: device_name.clone(),
            last_content: String::new(),
            last_update: chrono::Utc::now(),
        });

        if !device_name.is_empty() {
            session.device_name = device_name;
        }
        session.last_content = content;
        session.last_update = chrono::Utc::now();
    }

    pub async fn get_device_session(&self, device_id: &str) -> Option<DeviceSession> {
        let sessions = self.device_sessions.lock().await;
        sessions.get(device_id).cloned()
    }
}
