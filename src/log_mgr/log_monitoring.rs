use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use notify::event::{EventKind, CreateKind, ModifyKind};
use std::{path::Path,fs};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;
use std::path::PathBuf;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde_json::Value;
use tokio::sync::broadcast;
use crate::log_mgr::rust_server::WsEventTx;

#[derive(Debug)]
pub enum WatchCommand {
    Add(PathBuf),
    Remove(PathBuf),
}
struct TailState {
    path: PathBuf,
    offset: u64,
    line_number: usize, // Track line numbers
}

pub struct LogContextData {
    pub(crate) filters: HashMap<PathBuf, String>,
    pub(crate) notifies: HashMap<PathBuf, String>,
}


pub(crate) fn load_log_contents(log_path: &Path) -> String{

    println!("In file {}", log_path.display());

    let contents = fs::read_to_string(log_path)
        .expect("error reading log file");

    return contents;
}


pub fn start_watcher_manager(cmd_rx: Receiver<WatchCommand>, log_tx: broadcast::Sender<WsEventTx>, context: Arc<Mutex<LogContextData>>) -> thread::JoinHandle<()> {

    thread::spawn(move || {
        let (event_tx, event_rx) = std::sync::mpsc::channel();
        let mut watchers: HashMap<PathBuf, RecommendedWatcher> = HashMap::new();
        let mut states: HashMap<PathBuf, TailState> = HashMap::new();

        loop {
            while let Ok(cmd) = cmd_rx.try_recv() {
                match cmd {
                    WatchCommand::Add(path) => {
                        if watchers.contains_key(&path) {
                            continue;
                        }
                        let mut old_lines= 0;

                        if path.exists() {
                            old_lines = send_old_log_lines(&path, &log_tx);
                        }

                        println!("Watching {:?}", path);

                        if path
                            .extension()
                            .and_then(|e| e.to_str())
                            .map_or(true, |e| !e.eq_ignore_ascii_case("json"))
                        {
                            let tx = event_tx.clone();
                            let mut watcher = match RecommendedWatcher::new(
                                move |res| {
                                    if let Ok(event) = res {
                                        let _ = tx.send(event);
                                    }
                                },
                                notify::Config::default(),
                            ) {
                                Ok(w) => w,
                                Err(e) => {
                                    eprintln!("Failed to create watcher for {}: {}", path.display(), e);
                                    continue;
                                }
                            };

                            if let Err(e) = watcher.watch(&path, RecursiveMode::Recursive) {
                                eprintln!("Failed to watch {}: {}", path.display(), e);
                                continue;
                            }

                            watchers.insert(path.clone(), watcher);

                            let offset = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                            states.insert(path.clone(), TailState {
                                path: path.clone(),
                                offset,
                                line_number: old_lines,
                            });
                        }
                    }

                    WatchCommand::Remove(path) => {
                        watchers.remove(&path);
                        states.remove(&path);
                        println!("Stopped watching {:?}", path);
                    }

                }
            }

            while let Ok(event) = event_rx.try_recv() {

                match event.kind {
                    EventKind::Modify(ModifyKind::Data(_)) => {
                        println!("File modified {:?}", event.paths);
                        for path in &event.paths {
                            let state = states.entry(path.clone()).or_insert_with(|| {
                                let offset = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                                TailState { path: path.clone(), offset, line_number: 0 }
                            });

                            match tail_new_data(state) {
                                Ok(new_data) => {
                                    for line in new_data.lines() {
                                        //let _ = log_tx.send((path.to_string_lossy().to_string(), line.to_string()));
                                        LogContextData::on_event_modified(&context.lock().unwrap(), &path, line, &log_tx);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to read {}: {}", path.display(), e);
                                }
                            }
                        }
                    }

                    EventKind::Create(CreateKind::File) => {
                        for path in &event.paths {
                            println!("File created {:?}", path);
                        }
                    }

                    _ => {
                        // ignore other events
                    }
                }

            }

            thread::sleep(Duration::from_millis(100));
        }
    })
}


fn tail_new_data(state: &mut TailState) -> std::io::Result<String> {

    let mut file = match File::open(&state.path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Cannot open {}: {}", state.path.display(), e);
            return Ok(String::new());
        }
    };

    let len = match file.metadata() {
        Ok(m) => m.len(),
        Err(e) => {
            eprintln!("Cannot get metadata for {}: {}", state.path.display(), e);
            return Ok(String::new());
        }
    };

    if len < state.offset {
        state.offset = 0;
        state.line_number = 0;
    }

    file.seek(SeekFrom::Start(state.offset))?;

    let mut buf = String::new();
    if let Err(e) = file.read_to_string(&mut buf) {
        eprintln!("Failed to read {}: {}", state.path.display(), e);
        return Ok(String::new());
    }

    state.offset = len;

    let mut numbered_buf = String::new();
    for line in buf.lines() {
        state.line_number += 1;
        numbered_buf.push_str(&format!("{}: {}\n", state.line_number, line));
    }


    Ok(numbered_buf)
}

pub fn send_old_log_lines(log_path: &Path, log_tx: &broadcast::Sender<WsEventTx>)-> usize {

    let contents = match fs::read_to_string(log_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to read {}: {}", log_path.display(), e);
            return 0;
        }
    };

    let text = if log_path.extension().and_then(|e| e.to_str()) == Some("json") {
        let v: Value = serde_json::from_str(&contents).unwrap_or_default();
        serde_json::to_string_pretty(&v).unwrap_or_default()
    } else {
        contents
    };

    let mut keep_line_nr = 0;
    let mut batch = Vec::with_capacity(200);

    for (i, line) in text.lines().enumerate() {
        batch.push(format!("{}: {}", i + 1, line));
        keep_line_nr = i + 1;

        if batch.len() == 200 {
            let _ = log_tx.send(WsEventTx::LogBatch {
                path: log_path.to_string_lossy().to_string(),
                lines: batch,
            });
            batch = Vec::with_capacity(200);
        }
    }

    // send remaining lines
    if !batch.is_empty() {
        let _ = log_tx.send(WsEventTx::LogBatch {
            path: log_path.to_string_lossy().to_string(),
            lines: batch,
        });
    }

    return keep_line_nr;
}

/*Filtering and notifications */
impl LogContextData {
    pub fn set_filter(&mut self, paths: Vec<PathBuf>, pattern: String) {
        let path = paths[0].clone();
        println!("set filter for path {} with pattern {}", path.display(), pattern);
        self.filters.insert(path, pattern);
    }

    pub fn remove_filter(&mut self, path: PathBuf) {
        self.filters.remove(&path);

    }

    pub fn set_notification(&mut self, paths: Vec<PathBuf>, pattern: String) {

        println!("set notification for path {} with pattern {}", paths[0].display(), pattern);
        let path = paths[0].clone();
        self.notifies.insert(path, pattern);
    }

    pub fn remove_notification(&mut self, path: PathBuf) {

        self.notifies.remove(&path);
    }

    fn on_event_modified(&self, path: &PathBuf, line: &str, log_tx: &broadcast::Sender<WsEventTx>) {

        if let Some(pattern) = self.notifies.get(path) {
            if line.to_lowercase().contains(&pattern.to_lowercase()) {
                let to_send = format!("NOTIFICATION: {}", line);
                let _ = log_tx.send(WsEventTx::SearchResult {
                    path: path.to_string_lossy().to_string(),
                    lines: vec![to_send],
                });
            }
        }

        let (_line_number, content) = match line.split_once(": ") {
            Some((num, rest)) => (num, rest),
            None => ("", line),
        };

        if let Some(filter_pattern) = self.filters.get(path) {
            if !content.to_lowercase().contains(&filter_pattern.to_lowercase()) {
                return;
            }
        }

        let _ = log_tx.send(WsEventTx::Log {
            path: path.to_string_lossy().to_string(),
            line: line.to_string()
        });
    }
}
