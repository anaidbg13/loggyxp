use std::fs;
use notify::{Event, RecursiveMode, Result, Watcher};
use std::{path::Path, sync::mpsc};

pub(crate) fn read_and_print_log(log_path: &Path) -> String{

    println!("In file {}", log_path.display());

    let contents = fs::read_to_string(log_path)
        .expect("");

   // println!("Log:\n{contents}");
    return contents;
}

pub fn file_monitoring() -> Result<()> {


    let mut watcher = notify::recommended_watcher(event_fn)?;


    watcher.watch(Path::new("./tests"), RecursiveMode::Recursive)?;


    Ok(())
}

fn event_fn(res: Result<notify::Event>) {
    match res {
        Ok(event) => {
            println!("event: {:?}", event)
        },
        Err(e) => println!("watch error: {:?}", e),
    }
}
