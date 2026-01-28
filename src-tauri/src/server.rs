use crate::config::Config;
use crate::hub::Hub;
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
use tower_http::cors::CorsLayer;

pub struct Server {
    config: Config,
    hub: Arc<Hub>,
}

impl Server {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            hub: Arc::new(Hub::new()),
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let app = Router::new()
            .route("/", any(index_handler))
            .route("/ws", any(websocket_handler))
            .route("/type", any(type_handler))
            .layer(CorsLayer::permissive())
            .with_state(self.hub.clone());

        let addr = format!("0.0.0.0:{}", self.config.port);
        let listener = TcpListener::bind(&addr).await?;

        axum::serve(listener, app).await?;
        Ok(())
    }
}

async fn index_handler() -> impl IntoResponse {
    Html(include_str!("../assets/mobile.html"))
}

async fn type_handler(Json(payload): Json<Value>) -> impl IntoResponse {
    if let Some(text) = payload.get("text").and_then(|v| v.as_str()) {
        crate::virtual_keyboard::paste_text(text).await.ok();
    }
    Json(serde_json::json!({"success": true}))
}

async fn websocket_handler(
    State(hub): State<Arc<Hub>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(hub, socket))
}

async fn handle_socket(hub: Arc<Hub>, socket: WebSocket) {
    use futures_util::{SinkExt, StreamExt};

    let (mut sender, mut receiver) = socket.split();

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<axum::extract::ws::Message>();

    let mut client_id: Option<String> = None;
    let mut client_type: Option<crate::types::ClientType> = None;

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
                    if let Ok(ws_msg) = serde_json::from_str::<crate::types::WSMessage>(&text) {
                        match ws_msg.msg_type {
                            crate::types::MessageType::Connect => {
                                if let Some(data) = ws_msg.data.as_object() {
                                    if let Some(type_str) = data.get("type").and_then(|v| v.as_str()) {
                                        let parsed_type = match type_str {
                                            "desktop" => crate::types::ClientType::Desktop,
                                            "mobile" => crate::types::ClientType::Mobile,
                                            _ => continue,
                                        };

                                        let id = data.get("device_id")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or(&uuid::Uuid::new_v4().to_string())
                                            .to_string();

                                        match parsed_type {
                                            crate::types::ClientType::Desktop => {
                                                hub_clone.close_old_desktop().await;
                                                hub_clone.register_desktop(tx.clone()).await;
                                            }
                                            crate::types::ClientType::Mobile => {
                                                if let Some(session) = hub_clone.get_device_session(&id).await {
                                                    if !session.last_content.is_empty() {
                                                        let restore_msg = crate::types::WSMessage {
                                                            msg_type: crate::types::MessageType::Preview,
                                                            data: serde_json::json!({
                                                                "text": session.last_content,
                                                                "length": session.last_content.len(),
                                                                "device_name": session.device_name,
                                                                "device_id": session.device_id,
                                                                "restore": true
                                                            }),
                                                            timestamp: Some(chrono::Utc::now().timestamp()),
                                                            client_id: Some(id.clone()),
                                                        };
                                                        hub_clone.send_to_desktop(restore_msg).await.ok();
                                                    }
                                                }
                                                hub_clone.register_mobile(id.clone(), tx.clone()).await;
                                            }
                                        }

                                        client_id = Some(id.clone());
                                        client_type = Some(parsed_type);
                                    }
                                }
                            }
                            crate::types::MessageType::Preview => {
                                if let Some(id) = &client_id {
                                    if let Some(data) = ws_msg.data.as_object() {
                                        let text = data.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                        let d_name = data.get("device_name")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .to_string();

                                        hub_clone.update_device_session(id.clone(), d_name, text).await;
                                    }
                                }
                                hub_clone.send_to_desktop(ws_msg).await.ok();
                            }
                            crate::types::MessageType::Sync => {
                                hub_clone.broadcast_to_mobiles(ws_msg).await;
                            }
                            crate::types::MessageType::Send => {
                                if let Some(text_val) = ws_msg.data.get("text").and_then(|v| v.as_str()) {
                                    crate::virtual_keyboard::paste_text(text_val).await.ok();
                                }
                            }
                            crate::types::MessageType::Clear => {
                                hub_clone.send_to_desktop(ws_msg).await.ok();
                            }
                        }
                    }
                }
                axum::extract::ws::Message::Close(_) => break,
                _ => {}
            }
        }
        if let (Some(id), Some(ctype)) = (client_id, client_type) {
            match ctype {
                crate::types::ClientType::Desktop => {
                    hub_clone.unregister_desktop().await;
                }
                crate::types::ClientType::Mobile => {
                    hub_clone.unregister_mobile(&id).await;
                }
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = receive_task => {},
    }
}
