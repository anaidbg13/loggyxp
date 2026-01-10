use axum::{response::Html, routing::get, Router, extract::Json, extract::State};
use axum::response::IntoResponse;
use std::{net::SocketAddr, sync::Arc, thread};
use tokio::net::TcpListener;
use serde::Deserialize;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};

#[derive(Deserialize)]
struct PathRequest {
    path: String,
}
#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
enum ClientMessage {
    #[serde(rename = "watch_paths")]
    WatchPaths { paths: Vec<String> },

    #[serde(rename = "set_pattern")]
    SetPattern { pattern: String },

    #[serde(rename = "set_notify")]
    SetNotify { enabled: bool },

    #[serde(rename = "start_tailing")]
    StartTailing,

    #[serde(rename = "stop_tailing")]
    StopTailing,
}


fn load_html(path: &str) -> Html<String> {
    let html = std::fs::read_to_string(path)
        .unwrap_or_else(|_| "<h1>File not found</h1>".to_string());
    Html(html)
}

pub fn run_server() {
    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    let html_path = "static/dashboard.html";

    let Html(html) = load_html(html_path);
    let html = Arc::new(html);

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async move {
        let html_for_handler = html.clone();

        let app = Router::new().route(
            "/",
            get(move || {
                let html = html_for_handler.clone();
                async move { Html((*html).clone()) }
            }),

        )
            .route("/ws", get(ws_handler));;

        let listener = TcpListener::bind(addr).await.unwrap();
        println!("HTTP server listening on http://{}/", addr);

        axum::serve(listener, app).await.unwrap();
    });
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    // This tells Axum: "When the connection is upgraded to WS, run handle_socket"
    ws.on_upgrade(handle_socket)
}
async fn handle_socket(mut socket: WebSocket) {
    while let Some(Ok(msg)) = socket.recv().await {
        if let Message::Text(text) = msg {
            match serde_json::from_str::<ClientMessage>(&text) {
                Ok(ClientMessage::WatchPaths { paths }) => {
                    println!("Paths: {:?}", paths);
                }

                Ok(ClientMessage::SetPattern { pattern }) => {
                    println!("Pattern: {}", pattern);
                }

                Ok(ClientMessage::SetNotify { enabled }) => {
                    println!("Notify: {}", enabled);
                }

                Ok(ClientMessage::StartTailing) => {
                    println!("Start tailing");
                }

                Ok(ClientMessage::StopTailing) => {
                    println!("⏹ Stop tailing");
                }

                Err(e) => {
                    eprintln!("Invalid WS message: {}", e);
                }
            }
        }
    }
}


