use std::fs;
use std::path::Path;

pub(crate) fn read_and_print_log(log_path: &Path) -> String{

    println!("In file {}", log_path.display());

    let contents = fs::read_to_string(log_path)
        .expect("");

    println!("Log:\n{contents}");
    return contents;
}