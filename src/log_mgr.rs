use std::path::{Path, PathBuf};
use std::thread;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde_json::Value;
use tokio::sync::broadcast;
use search_engine::search_input_pattern;
use log_monitoring::start_watcher_manager;
use crate::log_mgr::log_monitoring::LogContextData;
use crate::log_mgr::rust_server::WsEventTx;

pub mod log_monitoring;
pub mod search_engine;
pub mod rust_server;
pub mod log_context_data;

pub fn main() {
    println!("main");

    let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();
    let (log_tx, _log_rx) = tokio::sync::broadcast::channel::<WsEventTx>(8192);

    let context = Arc::new(Mutex::new(LogContextData {
        filters: HashMap::new(),
        notifies: HashMap::new(),
    }));

    let context_for_watcher = Arc::clone(&context);
    let context_for_server = Arc::clone(&context);

    // Start server
    thread::spawn({
        let cmd_tx = cmd_tx.clone();
        let log_tx = log_tx.clone();
        move || {
            rust_server::run_server(cmd_tx, log_tx, context_for_server);
        }
    });

    // Start watcher
    let _ = start_watcher_manager(cmd_rx, log_tx, context_for_watcher);

    loop {
        std::thread::park();
    }
}

pub fn get_content(paths: &Vec<PathBuf>) -> String
{
    let log_path = Path::new(paths[0].to_str().unwrap());
    let content = log_monitoring::load_log_contents(log_path);

    if paths[0].exists() {
        let text = if paths[0].extension().and_then(|e| e.to_str()) == Some("json") {
            let v: Value = serde_json::from_str(&content).unwrap_or_default();
            serde_json::to_string_pretty(&v).unwrap_or_default()
        } else {
            content
        };
        return text;
    }
    String::new()
}

fn call_search_string(log_tx: &broadcast::Sender<WsEventTx>, pattern: &String, paths: Vec<PathBuf>)
{
    let content = get_content(&paths);
    let lines = search_engine::search_string(&content, &pattern);
    let _ = log_tx.send(WsEventTx::SearchResult {
        path: paths[0].to_string_lossy().to_string(),
        lines: lines.clone(),
    });
}

fn get_search_input_with_regex(log_tx: &broadcast::Sender<WsEventTx>, re_pattern: &String, paths: Vec<PathBuf>)
{
    let content = get_content(&paths);

    let  matches = search_input_pattern(&content,&re_pattern);

        let _ = log_tx.send(WsEventTx::SearchResult {
            path: paths[0].to_string_lossy().to_string(),
            lines: matches,
        });

}

