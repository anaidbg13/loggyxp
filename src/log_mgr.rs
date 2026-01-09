use std::path::Path;
use std::io;
use search_engine::search_input_pattern;
use log_monitoring::WatchCommand;

pub mod log_monitoring;
pub mod search_engine;
pub mod log_filtering;
pub mod log_notification;

pub fn function() {
    println!("inside log_mgr");
    get_content();
}

pub(crate) fn start_live_monitoring(cmd_tx: std::sync::mpsc::Sender<WatchCommand>) {
    std::thread::spawn(move || {
        use std::time::Duration;

        let paths = vec![
            "/tmp/dummyLogs/demo.txt".into(),
        ];

        for path in paths {
            println!("📨 Sending watch request: {:?}", path);
            cmd_tx.send(WatchCommand::Add(path)).unwrap();
            std::thread::sleep(Duration::from_secs(2));
        }
    });
}

/*log_monitoring functions*/
pub fn get_content()
{
    let log_path = Path::new("./tests/logs_for_testing/json1.json");
    let content = log_monitoring::read_and_print_log(log_path);
    //let content_2 = content.clone();
    //call_search_string(&content);
    //call_filter_lines(&content_2);
    get_search_input_with_regex(&content);

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

fn call_search_string(content: &String, pattern: &String) -> bool
{
    //let searched_word =  String::from("nobody");
    let found = search_engine::search_string(&content, &pattern);
    println!("Found: {}", found);
    //println!("Word count: {}", search_engine::pattern_frequency(&content, pattern));
    return found;
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
        //after checking validity, remove last index
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
    let found = call_search_string(&content, &pattern);
    if found {
        log_notification::notify_user();
    }
}
