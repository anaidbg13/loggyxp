use std::path::PathBuf;
use tokio::sync::broadcast;
use crate::log_mgr::LogContextData;
use crate::log_mgr::rust_server::WsEventTx;


impl LogContextData {
    // Set a filter pattern for the first path in the list
    pub fn set_filter(&mut self, paths: Vec<PathBuf>, pattern: String) {
        let path = paths[0].clone();
        println!("set filter for path {} with pattern {}", path.display(), pattern);
        self.filters.insert(path, pattern);
    }

    // Remove filter for a specific path
    pub fn remove_filter(&mut self, path: PathBuf) {
        self.filters.remove(&path);
    }

    // Set a notification pattern for the first path in the list
    pub fn set_notification(&mut self, paths: Vec<PathBuf>, pattern: String) {
        println!("set notification for path {} with pattern {}", paths[0].display(), pattern);
        let path = paths[0].clone();
        self.notifies.insert(path, pattern);
    }

    // Remove notification for a specific path
    pub fn remove_notification(&mut self, path: PathBuf) {
        self.notifies.remove(&path);
    }

    // Called when a log file is modified; sends notifications and filtered log lines
    pub fn on_event_modified(&self, path: &PathBuf, line: &str, log_tx: &broadcast::Sender<WsEventTx>) {
        // Check if notification pattern matches the line
        if let Some(pattern) = self.notifies.get(path) {
            if line.to_lowercase().contains(&pattern.to_lowercase()) {
                let to_send = format!("NOTIFICATION: {}", line);
                let _ = log_tx.send(WsEventTx::SearchResult {
                    path: path.to_string_lossy().to_string(),
                    lines: vec![to_send],
                });
            }
        }

        // Split line into line number and content
        let (_line_number, content) = match line.split_once(": ") {
            Some((num, rest)) => (num, rest),
            None => ("", line),
        };

        // If filter is set, only send lines that match the filter pattern
        if let Some(filter_pattern) = self.filters.get(path) {
            if !content.to_lowercase().contains(&filter_pattern.to_lowercase()) {
                return;
            }
        }

        // Send the log line to clients
        let _ = log_tx.send(WsEventTx::Log {
            path: path.to_string_lossy().to_string(),
            line: line.to_string()
        });
    }
}