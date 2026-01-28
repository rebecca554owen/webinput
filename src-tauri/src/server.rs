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
        println!("服务器运行在: {}", addr);

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

    let device_id = uuid::Uuid::new_v4().to_string();
    hub.register_mobile(device_id.clone(), tx).await;

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
                            crate::types::MessageType::Preview => {
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
                            _ => {}
                        }
                    }
                }
                axum::extract::ws::Message::Close(_) => break,
                _ => {}
            }
        }
        hub_clone.unregister_mobile(&device_id).await;
    });

    tokio::select! {
        _ = send_task => {},
        _ = receive_task => {},
    }
}
