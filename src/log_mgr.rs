use std::path::{Path, PathBuf};
use std::{io, thread};
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};
use search_engine::search_input_pattern;
use log_monitoring::WatchCommand;
use log_monitoring::start_watcher_manager;
use crate::log_mgr::rust_server::WsEventTx;

pub mod log_monitoring;
pub mod search_engine;
pub mod log_filtering;
pub mod log_notification;
mod rust_server;

pub fn main() {
    println!("main");

    let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();
    let (log_tx, _log_rx) = tokio::sync::broadcast::channel::<WsEventTx>(1024);

    // Start server
    thread::spawn({
        let cmd_tx = cmd_tx.clone();
        let log_tx = log_tx.clone();
        move || {
            rust_server::run_server(cmd_tx, log_tx);
        }
    });

    // Start watcher
    let _ = start_watcher_manager(cmd_rx, log_tx);

    loop {
        std::thread::park();
    }
}


pub fn function() {
    println!("inside log_mgr");
    //get_content();
}

pub fn start_live_monitoring(cmd_tx: std::sync::mpsc::Sender<WatchCommand>, paths: Vec<PathBuf>) {
    std::thread::spawn(move || {
        use std::time::Duration;

        for path in paths {
            println!("📨 Sending watch request: {:?}", path);
            cmd_tx.send(WatchCommand::Add(path)).unwrap();
            std::thread::sleep(Duration::from_secs(2));
        }
    });
}

pub fn remove_live_monitoring(cmd_tx: std::sync::mpsc::Sender<WatchCommand>, paths: Vec<PathBuf>) {

    for path in paths {
        println!("Removing watch request: {:?}", path);
        cmd_tx.send(WatchCommand::Remove(path)).unwrap();
        std::thread::sleep(Duration::from_secs(2));
    }
}

/*log_monitoring functions*/
pub fn get_content(paths: &Vec<PathBuf>) -> String
{

    if paths[0].exists() {
        println!("path exists");
        let log_path = Path::new(paths[0].to_str().unwrap());
        let content = log_monitoring::read_and_print_log(log_path);
        return content;
    }
    String::new()
}



/*log_filtering functions*/
fn call_filter_lines(content: &String, word: &String)
{
    log_filtering::parse_json(&content).expect("TODO: panic message");
    let keys = log_filtering::filter_by_key_json(&content, &word);
    let content_in = log_filtering::filter_lines(&content, &word);
    println!("filtered lines with word -{}-: \n{}",word, content_in);
    println!("Matches found for key {} : {} ",word, keys);

}

/*Search_engine functions*/

fn call_search_string(log_tx: &broadcast::Sender<WsEventTx>, pattern: &String, paths: Vec<PathBuf>)
{
    let content = get_content(&paths);
    let lines = search_engine::search_string(&content, &pattern);
    println!("lines with pattern {}: {:?}",pattern, lines);

    let _ = log_tx.send(WsEventTx::SearchResult {
        path: paths[0].to_string_lossy().to_string(),
        lines: lines.clone(),
    });
}
fn get_search_input_with_regex(content: &String)
{
    //read users input and format it
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let re_pattern: String = input.trim().parse().unwrap();

    let mut matches = search_input_pattern(&content,&re_pattern);

    if matches.last().unwrap().to_string() == "valid"
    {
        //after checking validity, remove last_index
        let last_index = matches.len() - 1;
        matches.remove(last_index);

        //remove duplicates
        matches.sort();
        matches.dedup();

            for m in matches {
                call_filter_lines(&content, &m);
            }
    }
    else
    {

    }


}

pub fn check_patterns(log_path: &Path) {

    let pattern = String::from("bbbbb");
    let content = log_monitoring::read_only_log(log_path);
    //let found = call_search_string(&content, pattern);
    // if found {
    //     log_notification::notify_user();
    // }
}
