use axum::{response::Html, routing::get, Router, extract::Json, extract::State};
use axum::response::IntoResponse;
use std::{net::SocketAddr, sync::Arc, path::PathBuf, sync::Mutex};
use tokio::net::TcpListener;
use serde::Deserialize;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use std::sync::mpsc::Sender;
use crate::log_mgr::log_monitoring::WatchCommand;
use futures_util::{StreamExt, SinkExt};
use tokio::sync::broadcast;

#[derive(Deserialize)]
struct PathRequest {
    path: String,
}

#[derive(Clone)]
struct AppState {
    cmd_tx: Sender<WatchCommand>,
    log_tx: broadcast::Sender<(String, String)>,
}

pub struct WsClient {
    tx: Sender<String>,
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
    StopTailing { paths: Vec<String> },
}


fn load_html(path: &str) -> Html<String> {
    let html = std::fs::read_to_string(path)
        .unwrap_or_else(|_| "<h1>File not found</h1>".to_string());
    Html(html)
}

pub fn run_server(cmd_tx: Sender<WatchCommand>, log_tx: tokio::sync::broadcast::Sender<(String, String)>) {
    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    let html_path = "static/dashboard.html";

    let Html(html) = load_html(html_path);
    let html = Arc::new(html);
    let state = AppState {
        cmd_tx,
        log_tx: log_tx.clone(),
    };



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

            .route(
                "/ws",
                get(move |ws: WebSocketUpgrade, State(state): State<AppState>| {
                    let log_rx = state.log_tx.subscribe();
                    ws_handler(ws, State(state), log_rx)
                }),
            )
            .with_state(state);


        let listener = TcpListener::bind(addr).await.unwrap();
        println!("HTTP server listening on http://{}/", addr);

        axum::serve(listener, app).await.unwrap();
    });
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    mut log_rx: broadcast::Receiver<(String, String)>
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, log_rx))
}

async fn handle_socket(
    mut socket: WebSocket,
    state: AppState,
    mut log_rx: broadcast::Receiver<(String, String)>
) {

    let (mut ws_tx, mut ws_rx) = socket.split();

    tokio::spawn(async move {
        while let Ok((path, line)) = log_rx.recv().await {
            let msg = serde_json::json!({
                "type": "log",
                "path": path,
                "line": line
            });
            let _ = ws_tx.send(Message::Text(msg.to_string().into())).await;
        }
    });

    while let Some(Ok(msg)) = ws_rx.next().await {
        if let Message::Text(text) = msg {
            match serde_json::from_str::<ClientMessage>(&text) {
                Ok(ClientMessage::WatchPaths { paths }) => {
                    println!("Paths: {:?}", paths);
                    let paths_buf = paths.into_iter()
                        .map(PathBuf::from)
                        .collect();
                    state.cmd_tx.send(WatchCommand::Add(paths_buf)).expect("failed to create watcher");
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
                Ok(ClientMessage::StopTailing { paths }) => {
                    println!("Stop tailing");
                    let paths_buf = paths.into_iter()
                        .map(PathBuf::from)
                        .collect();
                    state.cmd_tx.send(WatchCommand::Remove(paths_buf)).expect("failed to remove watcher");
                }
                Err(e) => {
                    eprintln!("Invalid WS message: {}", e);
                }
            }
        }
    }
}


