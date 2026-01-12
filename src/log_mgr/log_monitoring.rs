use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use notify::event::{EventKind, CreateKind, ModifyKind, RemoveKind};
use std::{path::Path,fs};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;
use std::path::PathBuf;
use crate::log_mgr;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::collections::HashMap;
use tokio::sync::broadcast;
use tokio::sync::mpsc::UnboundedSender;
use crate::log_mgr::rust_server::WsEventTx;

#[derive(Debug)]
pub enum WatchCommand {
    Add(PathBuf),
    Remove(PathBuf),
    Shutdown,
}
struct TailState {
    path: PathBuf,
    offset: u64,
    line_number: usize, // Track line numbers
}


pub(crate) fn read_and_print_log(log_path: &Path) -> String{

    println!("In file {}", log_path.display());

    let contents = fs::read_to_string(log_path)
        .expect("");

    println!("Log:\n{contents}");
    return contents;
}

pub fn read_only_log(log_path: &Path) -> String{

    let contents = fs::read_to_string(log_path).expect("error reading log file");

    return contents;
}



pub fn start_watcher_manager(cmd_rx: Receiver<WatchCommand>, log_tx: broadcast::Sender<WsEventTx>) -> thread::JoinHandle<()> {

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

                        if path.exists() {
                            send_old_log_lines(&path, &log_tx);
                        }

                        println!("Watching {:?}", path);

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

                        // Initialize tail state
                        let offset = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                        states.insert(path.clone(), TailState {
                            path: path.clone(),
                            offset,
                            line_number: 0,
                        });
                    }

                    WatchCommand::Remove(path) => {
                        watchers.remove(&path);
                        states.remove(&path);
                        println!("Stopped watching {:?}", path);
                    }

                    WatchCommand::Shutdown => {
                        println!("Watcher manager shutting down");
                        return;
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
                            println!("offset {}",state.offset );

                            match tail_new_data(state) {
                                Ok(new_data) => {
                                    for line in new_data.lines() {
                                        //let _ = log_tx.send((path.to_string_lossy().to_string(), line.to_string()));
                                        let _ = log_tx.send(WsEventTx::Log {
                                            path: path.to_string_lossy().to_string(),
                                            line: line.to_string()
                                        });
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to read {}: {}", path.display(), e);
                                }
                            }

                            // Optional: pattern checking
                            if let Some(first_path) = event.paths.get(0) {
                                if first_path.exists() {
                                    log_mgr::check_patterns(first_path);
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

    // Handle truncation / rotation
    if len < state.offset {
        state.offset = 0;
        state.line_number = 0;
    }
    println!("old offset {}",state.offset );
    // Move cursor to last read position
    file.seek(SeekFrom::Start(state.offset))?;

    let mut buf = String::new();
    if let Err(e) = file.read_to_string(&mut buf) {
        eprintln!("Failed to read {}: {}", state.path.display(), e);
        return Ok(String::new());
    }

    // Update offset
    state.offset = len;

    println!("new offset {}",state.offset );
    let mut numbered_buf = String::new();
    for line in buf.lines() {
        state.line_number += 1;
        numbered_buf.push_str(&format!("{}: {}\n", state.line_number, line));
    }


    Ok(numbered_buf)
}

pub fn send_old_log_lines(log_path: &Path, log_tx: &broadcast::Sender<WsEventTx>) {

    let contents = match fs::read_to_string(log_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to read {}: {}", log_path.display(), e);
            return;
        }
    };

    let mut count = 0;
    for (i, line) in contents.lines().enumerate() {
        let numbered_line = format!("{}: {}", i + 1, line);
        let _ = log_tx.send(WsEventTx::Log {
            path: log_path.to_string_lossy().to_string(),
            line: numbered_line,
        });
        count += 1;
        if count > 100 {
            count = 0;
            thread::sleep(Duration::from_millis(5));
        }
    }
}
