mod log_mgr;
use log_mgr::log_monitoring::start_watcher_manager;

fn main(){
    println!("main");

    let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();

    let _ = start_watcher_manager(cmd_rx);
    log_mgr::start_live_monitoring(cmd_tx.clone());
    log_mgr::function();

    loop{
        std::thread::park();
    }
}

