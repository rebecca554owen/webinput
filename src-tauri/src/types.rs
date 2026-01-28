use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Preview,
    Sync,
    Send,
    Clear,
    Connect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WSMessage {
    #[serde(rename = "type")]
    pub msg_type: MessageType,
    pub data: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ClientType {
    Desktop,
    Mobile,
}

#[derive(Debug, Clone)]
pub struct DeviceSession {
    pub device_id: String,
    pub device_name: String,
    pub last_content: String,
    pub last_update: chrono::DateTime<chrono::Utc>,
}
