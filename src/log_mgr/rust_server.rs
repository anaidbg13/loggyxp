use axum::{response::Html, routing::get, Router, extract::Json, extract::State};
use axum::response::IntoResponse;
use std::{net::SocketAddr, sync::Arc, path::PathBuf, sync::Mutex};
use tokio::net::TcpListener;
use serde::{Deserialize, Serialize};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use std::sync::mpsc::Sender;
use crate::log_mgr::log_monitoring::WatchCommand;
use futures_util::{StreamExt, SinkExt};
use tokio::sync::broadcast;
use crate::log_mgr;

#[derive(Deserialize)]
struct PathRequest {
    path: String,
}

#[derive(Clone)]
struct AppState {
    cmd_tx: Sender<WatchCommand>,
    log_tx: broadcast::Sender<WsEventTx>,
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
    SetPattern { paths: Vec<String>, pattern: String },

    #[serde(rename = "set_notify")]
    SetNotify { paths: Vec<String>, enabled: bool },

    #[serde(rename = "start_tailing")]
    StartTailing {paths: Vec<String> },

    #[serde(rename = "stop_tailing")]
    StopTailing { paths: Vec<String> },

    #[serde(rename = "search")]
    Search {
        paths: Vec<String>,
        pattern: String,
        regex: bool,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum WsEventTx {
    #[serde(rename = "log")]
    Log {
        path: String,
        line: String,
    },

    #[serde(rename = "search_result")]
    SearchResult {
        path: String,
        lines: Vec<usize>,
    },
}


fn load_html(path: &str) -> Html<String> {
    let html = std::fs::read_to_string(path)
        .unwrap_or_else(|_| "<h1>File not found</h1>".to_string());
    Html(html)
}

pub fn run_server(cmd_tx: Sender<WatchCommand>, log_tx: broadcast::Sender<WsEventTx>) {
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
    mut log_rx: broadcast::Receiver<WsEventTx>
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, log_rx))
}

async fn handle_socket(
    mut socket: WebSocket,
    state: AppState,
    mut log_rx: broadcast::Receiver<WsEventTx>
) {

    let (mut ws_tx, mut ws_rx) = socket.split();

    tokio::spawn(async move {
        while let Ok(event) = log_rx.recv().await {
            let json = serde_json::to_string(&event).unwrap();
            let _ = ws_tx
                .send(Message::Text(json.into()))
                .await;
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
                Ok(ClientMessage::SetPattern { paths,pattern }) => {
                    println!("Pattern: {}", pattern);
                }
                Ok(ClientMessage::SetNotify { paths, enabled }) => {
                    println!("Notify: {}", enabled);
                }
                Ok(ClientMessage::StartTailing { paths}) => {
                    println!("Start tailing");
                    let paths_buf = paths.into_iter()
                        .map(PathBuf::from)
                        .collect();
                    state.cmd_tx.send(WatchCommand::Add(paths_buf)).expect("failed to remove watcher");
                }
                Ok(ClientMessage::StopTailing { paths }) => {
                    println!("Stop tailing {}", paths.join(", "));
                    let paths_buf = paths.into_iter()
                        .map(PathBuf::from)
                        .collect();
                    state.cmd_tx.send(WatchCommand::Remove(paths_buf)).expect("failed to remove watcher");
                }
                Ok(ClientMessage::Search { paths, pattern, regex }) => {
                    println!(
                        "Search request: paths={:?}, pattern={}, regex={}",
                        paths, pattern, regex
                    );

                    let paths_buf: Vec<_> = paths
                        .into_iter()
                        .map(PathBuf::from)
                        .collect();

                    log_mgr::call_search_string(&state.log_tx, &pattern, paths_buf);
                }
                Err(e) => {
                    eprintln!("Invalid WS message: {}", e);
                }
            }
        }
    }
}


