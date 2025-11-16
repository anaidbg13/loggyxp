use std::path::Path;
pub mod log_monitoring;
pub mod search_engine;

pub fn function() {
    println!("inside log_mgr");
    get_content();
}

pub fn get_content()
{
    let log_path = Path::new("./tests/logs_for_testing/dummy_log1.txt");
    let content = log_monitoring::read_and_print_log(log_path);
    call_search_word(content);

}

fn call_search_word(content: String)
{
    let searched_word =  String::from("nobody");
    let content2 = content.clone();
    let pattern = searched_word.clone();
    let found = search_engine::search_word(content, searched_word);
    println!("Found: {}", found);
    println!("Word count: {}", search_engine::pattern_frequency(content2, pattern));
}
