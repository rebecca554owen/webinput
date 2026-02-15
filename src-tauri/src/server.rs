use crate::config::Config;
use crate::hub::Hub;
use crate::types::{ClientType, MessageType, WSMessage};
use axum::{
    extract::{
        State,
        ws::{WebSocket, WebSocketUpgrade},
    },
    response::{Html, IntoResponse},
    routing::any,
    Json, Router,
};
use serde_json::Value;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

const CLIENT_TYPE_DESKTOP: &str = "desktop";
const CLIENT_TYPE_MOBILE: &str = "mobile";
const DATA_TYPE_KEY: &str = "type";
const DATA_DEVICE_ID_KEY: &str = "device_id";
const DATA_TEXT_KEY: &str = "text";
const DATA_DEVICE_NAME_KEY: &str = "device_name";
const DATA_APPEND_ENTER_KEY: &str = "append_enter";
const DATA_RESTORE_KEY: &str = "restore";

pub struct Server {
    config: Config,
    hub: Arc<Hub>,
    auto_enter: Arc<Mutex<bool>>,
}

impl Server {
    pub fn new(config: Config, auto_enter: Arc<Mutex<bool>>) -> Self {
        Self {
            config,
            hub: Arc::new(Hub::new()),
            auto_enter,
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let app = Router::new()
            .route("/", any(index_handler))
            .route("/ws", any(websocket_handler))
            .route("/type", any(type_handler))
            .layer(CorsLayer::permissive())
            .with_state((self.hub.clone(), self.auto_enter.clone()));

        let addr = format!("0.0.0.0:{}", self.config.port);
        let listener = TcpListener::bind(&addr).await?;

        axum::serve(listener, app).await?;
        Ok(())
    }
}

async fn index_handler() -> impl IntoResponse {
    Html(include_str!("../assets/mobile.html"))
}

async fn type_handler(
    State((_hub, auto_enter)): State<(Arc<Hub>, Arc<Mutex<bool>>)>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some(text) = payload.get(DATA_TEXT_KEY).and_then(|v| v.as_str()) {
        let append_enter = payload.get(DATA_APPEND_ENTER_KEY)
            .and_then(|v| v.as_bool())
            .unwrap_or(*auto_enter.lock().await);
        if let Err(e) = crate::virtual_keyboard::paste_text(text, append_enter).await {
            eprintln!("[type_handler] Failed to paste text: {}", e);
        }
    }
    Json(serde_json::json!({"success": true}))
}

async fn websocket_handler(
    State((hub, _auto_enter)): State<(Arc<Hub>, Arc<Mutex<bool>>)>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(hub, socket))
}

async fn handle_socket(hub: Arc<Hub>, socket: WebSocket) {
    use futures_util::{SinkExt, StreamExt};

    let (mut sender, mut receiver) = socket.split();

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<axum::extract::ws::Message>();

    let mut client_id: Option<String> = None;
    let mut client_type: Option<ClientType> = None;

    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let hub_clone = hub.clone();
    let receive_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                axum::extract::ws::Message::Text(text) => {
                    if let Ok(ws_msg) = serde_json::from_str::<WSMessage>(&text) {
                        handle_client_message(
                            &hub_clone,
                            &tx,
                            ws_msg,
                            &mut client_id,
                            &mut client_type,
                        ).await;
                    }
                }
                axum::extract::ws::Message::Close(_) => break,
                _ => {}
            }
        }

        if let (Some(id), Some(ctype)) = (client_id, client_type) {
            unregister_client(&hub_clone, &id, ctype).await;
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = receive_task => {},
    }
}

async fn handle_client_message(
    hub: &Hub,
    tx: &tokio::sync::mpsc::UnboundedSender<axum::extract::ws::Message>,
    ws_msg: WSMessage,
    client_id: &mut Option<String>,
    client_type: &mut Option<ClientType>,
) {
    match ws_msg.msg_type {
        MessageType::Connect => {
            handle_connect_message(hub, tx, ws_msg, client_id, client_type).await;
        }
        MessageType::Preview => {
            handle_preview_message(hub, ws_msg, client_id).await;
        }
        MessageType::Sync => {
            hub.broadcast_to_mobiles(ws_msg).await;
        }
        MessageType::Send => {
            handle_send_message(ws_msg).await;
        }
        MessageType::Clear => {
            if let Err(e) = hub.send_to_desktop(ws_msg).await {
                eprintln!("[handle_client_message] Failed to send clear message: {}", e);
            }
        }
    }
}

async fn handle_connect_message(
    hub: &Hub,
    tx: &tokio::sync::mpsc::UnboundedSender<axum::extract::ws::Message>,
    ws_msg: WSMessage,
    client_id: &mut Option<String>,
    client_type: &mut Option<ClientType>,
) {
    if let Some(data) = ws_msg.data.as_object() {
        if let Some(type_str) = data.get(DATA_TYPE_KEY).and_then(|v| v.as_str()) {
            let parsed_type = match type_str {
                CLIENT_TYPE_DESKTOP => ClientType::Desktop,
                CLIENT_TYPE_MOBILE => ClientType::Mobile,
                _ => {
                    eprintln!("[handle_connect_message] Unknown client type: {}", type_str);
                    return;
                }
            };

            let id = data.get(DATA_DEVICE_ID_KEY)
                .and_then(|v| v.as_str())
                .unwrap_or(&uuid::Uuid::new_v4().to_string())
                .to_string();

            match parsed_type {
                ClientType::Desktop => {
                    hub.close_old_desktop().await;
                    hub.register_desktop(tx.clone()).await;
                }
                ClientType::Mobile => {
                    restore_device_session(hub, &id).await;
                    hub.register_mobile(id.clone(), tx.clone()).await;
                }
            }

            *client_id = Some(id);
            *client_type = Some(parsed_type);
        }
    }
}

async fn restore_device_session(hub: &Hub, device_id: &str) {
    if let Some(session) = hub.get_device_session(device_id).await {
        if !session.last_content.is_empty() {
            let restore_msg = WSMessage {
                msg_type: MessageType::Preview,
                data: serde_json::json!({
                    DATA_TEXT_KEY: session.last_content,
                    "length": session.last_content.len(),
                    DATA_DEVICE_NAME_KEY: session.device_name,
                    DATA_DEVICE_ID_KEY: session.device_id,
                    DATA_RESTORE_KEY: true
                }),
                timestamp: Some(chrono::Utc::now().timestamp()),
                client_id: Some(device_id.to_string()),
            };
            if let Err(e) = hub.send_to_desktop(restore_msg).await {
                eprintln!("[restore_device_session] Failed to send restore message: {}", e);
            }
        }
    }
}

async fn handle_preview_message(hub: &Hub, ws_msg: WSMessage, client_id: &Option<String>) {
    if let Some(id) = client_id {
        if let Some(data) = ws_msg.data.as_object() {
            let text = data.get(DATA_TEXT_KEY).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let d_name = data.get(DATA_DEVICE_NAME_KEY)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            hub.update_device_session(id.clone(), d_name, text).await;
        }
    }
    if let Err(e) = hub.send_to_desktop(ws_msg).await {
        eprintln!("[handle_preview_message] Failed to send preview message: {}", e);
    }
}

async fn handle_send_message(ws_msg: WSMessage) {
    if let Some(text_val) = ws_msg.data.get(DATA_TEXT_KEY).and_then(|v| v.as_str()) {
        let append_enter = ws_msg.data.get(DATA_APPEND_ENTER_KEY)
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if let Err(e) = crate::virtual_keyboard::paste_text(text_val, append_enter).await {
            eprintln!("[handle_send_message] Failed to paste text: {}", e);
        }
    }
}

async fn unregister_client(hub: &Hub, id: &str, ctype: ClientType) {
    match ctype {
        ClientType::Desktop => {
            hub.unregister_desktop().await;
        }
        ClientType::Mobile => {
            hub.unregister_mobile(id).await;
        }
    }
}
