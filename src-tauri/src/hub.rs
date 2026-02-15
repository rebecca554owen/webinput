//! WebSocket 连接管理中心
//!
//! 该模块提供 [`Hub`] 结构体，用于管理桌面端和手机端之间的 WebSocket 连接。
//! 支持单一桌面端连接、多手机端连接，以及设备会话的持久化管理。

use crate::types::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};

type ClientSender = mpsc::UnboundedSender<axum::extract::ws::Message>;

/// WebSocket 连接管理中心
///
/// 管理三种类型的连接和状态：
/// - `desktop`: 单一桌面端连接
/// - `mobiles`: 多个手机端连接（按设备 ID 索引）
/// - `device_sessions`: 设备会话信息（支持跨重连恢复内容）
pub struct Hub {
    desktop: Arc<RwLock<Option<ClientSender>>>,
    mobiles: Arc<RwLock<HashMap<String, ClientSender>>>,
    device_sessions: Arc<Mutex<HashMap<String, DeviceSession>>>,
}

impl Hub {
    /// 创建新的 Hub 实例
    ///
    /// # Returns
    ///
    /// 返回初始化后的 Hub，所有连接和会话均为空
    pub fn new() -> Self {
        Self {
            desktop: Arc::new(RwLock::new(None)),
            mobiles: Arc::new(RwLock::new(HashMap::new())),
            device_sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 注册桌面端连接
    ///
    /// # Arguments
    ///
    /// * `sender` - 用于向桌面端发送消息的 channel 发送端
    pub async fn register_desktop(&self, sender: ClientSender) {
        let mut desktop = self.desktop.write().await;
        *desktop = Some(sender);
    }

    /// 关闭旧的桌面端连接
    ///
    /// 如果存在已注册的桌面端连接，发送 Close 消息并断开连接。
    pub async fn close_old_desktop(&self) {
        let mut desktop = self.desktop.write().await;
        if let Some(old_sender) = desktop.take() {
            let _ = old_sender.send(axum::extract::ws::Message::Close(None));
        }
    }

    /// 注册手机端连接
    ///
    /// # Arguments
    ///
    /// * `id` - 设备唯一标识符
    /// * `sender` - 用于向该手机端发送消息的 channel 发送端
    pub async fn register_mobile(&self, id: String, sender: ClientSender) {
        let mut mobiles = self.mobiles.write().await;
        mobiles.insert(id, sender);
    }

    /// 注销桌面端连接
    ///
    /// 将桌面端连接设置为 None，表示已断开。
    pub async fn unregister_desktop(&self) {
        let mut desktop = self.desktop.write().await;
        *desktop = None;
    }

    /// 注销手机端连接
    ///
    /// # Arguments
    ///
    /// * `id` - 要注销的设备唯一标识符
    pub async fn unregister_mobile(&self, id: &str) {
        let mut mobiles = self.mobiles.write().await;
        mobiles.remove(id);
    }

    /// 向桌面端发送消息
    ///
    /// # Arguments
    ///
    /// * `msg` - 要发送的 WebSocket 消息
    ///
    /// # Returns
    ///
    /// 成功返回 `Ok(())`，序列化或发送失败返回错误
    pub async fn send_to_desktop(&self, msg: WSMessage) -> Result<(), Box<dyn std::error::Error>> {
        let desktop = self.desktop.read().await;
        if let Some(sender) = desktop.as_ref() {
            let json = serde_json::to_string(&msg)?;
            sender.send(axum::extract::ws::Message::Text(json))?;
        }
        Ok(())
    }

    /// 向所有手机端广播消息
    ///
    /// # Arguments
    ///
    /// * `msg` - 要广播的 WebSocket 消息
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

    /// 更新设备会话信息
    ///
    /// 如果设备会话不存在则创建新会话，否则更新现有会话的内容和时间戳。
    ///
    /// # Arguments
    ///
    /// * `device_id` - 设备唯一标识符
    /// * `device_name` - 设备名称（非空时更新）
    /// * `content` - 设备最后发送的内容
    pub async fn update_device_session(&self, device_id: String, device_name: String, content: String) {
        let mut sessions = self.device_sessions.lock().await;
        let session = sessions.entry(device_id.clone()).or_insert(DeviceSession {
            device_id,
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

    /// 获取设备会话信息
    ///
    /// # Arguments
    ///
    /// * `device_id` - 设备唯一标识符
    ///
    /// # Returns
    ///
    /// 如果设备会话存在，返回 `Some(DeviceSession)`，否则返回 `None`
    pub async fn get_device_session(&self, device_id: &str) -> Option<DeviceSession> {
        let sessions = self.device_sessions.lock().await;
        sessions.get(device_id).cloned()
    }
}
