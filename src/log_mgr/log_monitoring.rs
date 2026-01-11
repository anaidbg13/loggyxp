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

#[derive(Debug)]
pub enum WatchCommand {
    Add(PathBuf),
    Remove(PathBuf),
    Shutdown,
}
struct TailState {
    path: PathBuf,
    offset: u64,
}

pub struct LogEvent {
    pub path: String,
    pub line: String,
}



pub(crate) fn read_and_print_log(log_path: &Path) -> String{

    println!("In file {}", log_path.display());

    let contents = fs::read_to_string(log_path)
        .expect("");

   // println!("Log:\n{contents}");
    return contents;
}

pub fn read_only_log(log_path: &Path) -> String{

    let contents = fs::read_to_string(log_path).expect("error reading log file");

    return contents;
}



pub fn start_watcher_manager(cmd_rx: Receiver<WatchCommand>, log_tx: broadcast::Sender<(String, String)>) -> thread::JoinHandle<()> {

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

                        println!("Watching {:?}", path);

                        let tx = event_tx.clone();
                        let mut watcher = RecommendedWatcher::new(
                            move |res| {
                                if let Ok(event) = res {
                                    let _ = tx.send(event);
                                }
                            },
                            notify::Config::default(),
                        )
                            .expect("watcher create failed");

                        watcher
                            .watch(&path, RecursiveMode::Recursive)
                            .expect("watch failed");

                        watchers.insert(path, watcher);
                    }

                    WatchCommand::Remove(path) => {
                        //println!("Stop watching {:?}", path);
                        watchers.remove(&path);
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
                            let mut state = states.entry(path.clone()).or_insert_with(|| {
                                let offset = std::fs::metadata(&path)
                                    .map(|m| m.len())
                                    .unwrap_or(0);

                                TailState {
                                    path: path.clone(),
                                    offset,
                                }
                            });
                            println!("offset {}",state.offset );

                            let new_data = tail_new_data(&mut state).unwrap();

                            for line in new_data.lines() {
                                println!("TAIL ▶ {}", line);
                                let _ = log_tx.send((path.to_string_lossy().to_string(), line.to_string()));
                            }

                            log_mgr::check_patterns(&event.paths[0]);
                        }


                    }
                    EventKind::Create(CreateKind::File) => {
                        println!("File created {:?}", event.paths);

                    }

                    _ => {
                        // ignore other events
                    }
                }

            }

            thread::sleep(Duration::from_millis(1000));
        }
    })
}


fn tail_new_data(state: &mut TailState) -> std::io::Result<String> {

    let mut file = File::open(&state.path)?;

    let len = file.metadata()?.len();
    println!("len new {}",len );

    // Handle truncation / rotation
    if len < state.offset {
        state.offset = 0;
    }
    println!("old offset {}",state.offset );
    // Move cursor to last read position
    file.seek(SeekFrom::Start(state.offset))?;

    let mut buf = String::new();
    file.read_to_string(&mut buf)?;

    // Update offset
    state.offset = len;

    println!("new offset {}",state.offset );

    Ok(buf)
}

