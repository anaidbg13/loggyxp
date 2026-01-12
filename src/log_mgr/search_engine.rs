extern crate regex;
use regex::Regex;


pub fn search_string(content: &String, word: &String) -> Vec<usize> {
    let mut lines = Vec::new();

    if word.is_empty() {
        return lines;
    }

    let needle = word.to_lowercase();

    for (idx, line) in content.lines().enumerate() {
        if line.to_lowercase().contains(&needle) {
            lines.push(idx + 1);
        }
    }

    if lines.is_empty() {
        println!("'{}' was not found", word);
    } else {
        println!("'{}' found on lines {:?}", word, lines);
    }

    lines
}







pub fn search_input_pattern(content: &String, pattern: &String) -> Vec<String>
{

    let mut matches: Vec<String> = Vec::new();

    match Regex::new(pattern) {
        Ok(regex) => {
            // Collect all matches as strings
                matches = regex
                .find_iter(content)
                .map(|m| m.as_str().to_string())
                .collect();

            if !matches.is_empty() {
                println!("{} has {} matches", pattern, matches.len());
                matches.push(String::from("valid"));
                return matches;
            } else {
                println!("Could not find any matches");
                matches.push(String::from("invalid"));
                return matches;
            }
        }
        Err(_) => {
            println!("Invalid pattern");
            matches.push(String::from("invalid"));
            return matches;
        }
    }

    /*if let Ok(re) = Regex::new(&pattern) {
        println!("Regex is valid!");
    } else {
        eprintln!("Regex pattern is invalid");
    }*/
}

pub fn remove_duplicates(matches: &Vec<String>) -> Vec<String>
{
    let mut non_duplicates = matches.clone();

    non_duplicates.sort();
    non_duplicates.dedup();

    return non_duplicates;

}