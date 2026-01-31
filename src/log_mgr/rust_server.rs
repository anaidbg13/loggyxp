use axum::{response::Html, routing::get, Router, extract::State};
use axum::response::IntoResponse;
use std::{net::SocketAddr, sync::Arc, path::PathBuf, sync::Mutex};
use tokio::net::TcpListener;
use serde::{Deserialize, Serialize};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use std::sync::mpsc::Sender;
use crate::log_mgr::log_monitoring::{LogContextData, WatchCommand};
use futures_util::{StreamExt, SinkExt};
use tokio::sync::broadcast;
use crate::log_mgr;

// Shared application state for handlers
#[derive(Clone)]
struct AppState {
    cmd_tx: Sender<WatchCommand>, // Channel to send watch commands
    log_tx: broadcast::Sender<WsEventTx>, // Channel to broadcast log events
    context: Arc<Mutex<LogContextData>>, // Shared log context
}

// Messages received from the client via WebSocket
#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
enum ClientMessage {
    #[serde(rename = "watch_paths")]
    WatchPaths { paths: Vec<String> },

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

    #[serde(rename = "Filter_by")]
    FilterBy {
        paths: Vec<String>,
        pattern: String,
    },

    #[serde(rename = "Notify_when")]
    NotifyWhen {
        paths: Vec<String>,
        pattern: String,
    },

    #[serde(rename = "remove_filter")]
    RemoveFilter {
        paths: Vec<String>,
    },

    #[serde(rename = "remove_notification")]
    RemoveNotification {
        paths: Vec<String>,
    },
}

// Events sent to the client via WebSocket
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
        lines: Vec<String>,
    },

    #[serde(rename = "log_batch")]
    LogBatch {
        path: String,
        lines: Vec<String>,
    }
}

// Loads HTML file for dashboard
fn load_html(path: &str) -> Html<String> {
    let html = std::fs::read_to_string(path)
        .unwrap_or_else(|_| "<h1>File not found</h1>".to_string());
    Html(html)
}

// Starts the HTTP and WebSocket server
pub fn run_server(cmd_tx: Sender<WatchCommand>, log_tx: broadcast::Sender<WsEventTx>, context: Arc<Mutex<LogContextData>>) {
    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    let html_path = "static/dashboard.html";

    let Html(html) = load_html(html_path);
    let html = Arc::new(html);
    let state = AppState {
        cmd_tx,
        log_tx: log_tx.clone(),
        context: context.clone(),
    };

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async move {
        let html_for_handler = html.clone();

        // Set up routes for HTTP and WebSocket
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

        // Bind TCP listener and start server
        let listener = TcpListener::bind(addr).await.unwrap();
        println!("HTTP server listening on http://{}/", addr);

        axum::serve(listener, app).await.unwrap();
    });
}

// Handles WebSocket upgrade and delegates to socket handler
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    log_rx: broadcast::Receiver<WsEventTx>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, log_rx))
}

// Handles communication with a single WebSocket client
async fn handle_socket(
    socket: WebSocket,
    state: AppState,
    mut log_rx: broadcast::Receiver<WsEventTx>
) {

    let (mut ws_tx, mut ws_rx) = socket.split();

    // Spawn a task to send log events to the client
    tokio::spawn(async move {
        while let Ok(event) = log_rx.recv().await {
            let json = serde_json::to_string(&event).unwrap();
            let _ = ws_tx
                .send(Message::Text(json.into()))
                .await;
        }
    });

    // Main loop to receive and handle client messages
    while let Some(Ok(msg)) = ws_rx.next().await {
        if let Message::Text(text) = msg {
            match serde_json::from_str::<ClientMessage>(&text) {
                Ok(ClientMessage::WatchPaths { paths }) => {
                    // Add paths to watcher
                    let paths_buf = paths.into_iter()
                        .map(PathBuf::from)
                        .collect();
                    state.cmd_tx.send(WatchCommand::Add(paths_buf)).expect("failed to create watcher");
                }
                Ok(ClientMessage::StartTailing { paths}) => {
                    println!("Start tailing log(s)");
                    let paths_buf = paths.into_iter()
                        .map(PathBuf::from)
                        .collect();
                    state.cmd_tx.send(WatchCommand::Add(paths_buf)).expect("failed to create watcher");
                }
                Ok(ClientMessage::StopTailing { paths }) => {
                    println!("Stop tailing {}", paths.join(", "));
                    let paths_buf = paths.into_iter()
                        .map(PathBuf::from)
                        .collect();
                    state.cmd_tx.send(WatchCommand::Remove(paths_buf)).expect("failed to remove watcher");
                }
                Ok(ClientMessage::Search { paths, pattern, regex }) => {
                    // Perform search (regex or string)
                    let paths_buf: Vec<_> = paths
                        .into_iter()
                        .map(PathBuf::from)
                        .collect();
                    
                    if regex {
                        log_mgr::get_search_input_with_regex(&state.log_tx, &pattern, paths_buf);
                    }
                    else {
                        log_mgr::call_search_string(&state.log_tx, &pattern, paths_buf);
                    }
                }
                Ok(ClientMessage::FilterBy { paths, pattern}) => {
                    // Set filter for paths
                    let paths_buf: Vec<_> = paths
                        .into_iter()
                        .map(PathBuf::from)
                        .collect();

                    let mut ctx = state.context.lock().unwrap();
                    println!("Notify request: paths={:?}, pattern={}", paths_buf, pattern);
                    ctx.set_filter(paths_buf, pattern);

                }
                Ok(ClientMessage::NotifyWhen { paths, pattern}) => {
                    // Set notification for paths
                    let paths_buf: Vec<_> = paths
                        .into_iter()
                        .map(PathBuf::from)
                        .collect();

                    let mut ctx = state.context.lock().unwrap();
                    println!("Notify request: paths={:?}, pattern={}", paths_buf, pattern);
                    ctx.set_notification(paths_buf, pattern);

                }
                Ok(ClientMessage::RemoveFilter { paths }) => {
                    // Remove filter for path
                    let path: PathBuf = paths[0].clone().into();
                    let mut ctx = state.context.lock().unwrap();
                    ctx.remove_filter(path)
                }
                Ok(ClientMessage::RemoveNotification { paths }) => {
                    // Remove notification for path
                    let path: PathBuf = paths[0].clone().into();
                    let mut ctx = state.context.lock().unwrap();
                    ctx.remove_notification(path)
                }
                Err(e) => {
                    // Handle invalid client message
                    eprintln!("Invalid WS message: {}", e);
                }
            }
        }
    }
}


