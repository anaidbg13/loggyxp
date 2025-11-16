use std::path::Path;
pub mod log_monitoring;
pub mod search_engine;
pub mod log_filtering;

pub fn function() {
    println!("inside log_mgr");
    get_content();
}

pub fn get_content()
{
    let log_path = Path::new("./tests/logs_for_testing/dummy_log1.txt");
    let content = log_monitoring::read_and_print_log(log_path);
    let content_2 = content.clone();
    call_search_word(content);
    call_filter_lines(content_2);

}

fn call_search_word(content: String)
{
    let searched_word =  String::from("nobody");
    let content_in = content.clone();
    let content_in_2 = content.clone();
    let pattern = searched_word.clone();
    let found = search_engine::search_word(content_in, searched_word);
    println!("Found: {}", found);
    println!("Word count: {}", search_engine::pattern_frequency(content_in_2, pattern));
}

fn call_filter_lines(content: String)
{
    let filtered_content = content.clone();
    let pattern =  String::from("nobody");
    let pattern_2 = String::from("nobody");

    let content_in = log_filtering::filter_lines(filtered_content, pattern);

    println!("filtered lines with word -{}-: \n{}",pattern_2, content_in);
}