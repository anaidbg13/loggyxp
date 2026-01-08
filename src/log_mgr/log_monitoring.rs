use std::{path::Path, sync::mpsc,fs};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;
use std::path::PathBuf;

#[derive(Debug)]
pub enum WatchCommand {
    Add(PathBuf),
    Remove(PathBuf),
    Shutdown,
}

pub(crate) fn read_and_print_log(log_path: &Path) -> String{

    println!("In file {}", log_path.display());

    let contents = fs::read_to_string(log_path)
        .expect("");

   // println!("Log:\n{contents}");
    return contents;
}



pub fn start_watcher_manager(cmd_rx: Receiver<WatchCommand>, ) -> thread::JoinHandle<()> {

    thread::spawn(move || {
        let (event_tx, event_rx) = std::sync::mpsc::channel();
        let mut watchers: HashMap<PathBuf, RecommendedWatcher> = HashMap::new();
        //println!("Watcher manager started");

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

            // 2️⃣ Handle file events
            while let Ok(event) = event_rx.try_recv() {
                println!("Event: {:?}", event);
            }

            thread::sleep(Duration::from_millis(100));
        }
    })
}

