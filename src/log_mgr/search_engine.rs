extern crate regex;
use regex::Regex;


pub fn search_string(content: &String, word: &String) -> Vec<String> {
    let mut lines = Vec::new();

    if word.is_empty() {
        return lines;
    }

    let needle = word.to_lowercase();

    for (idx, line) in content.lines().enumerate() {
        if line.to_lowercase().contains(&needle) {
            lines.push(format!("{}: {}", idx + 1, line));
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
            matches = content
                .lines()
                .enumerate()
                .filter_map(|(idx, line)| {
                    regex.is_match(line)
                        .then(|| format!("{}: {}", idx + 1, line))
                })
                .collect();
            /*if !matches.is_empty() {
                return matches;
            } else {
                println!("Could not find any matches");
                //matches.push(String::from("Could not find any matches"));
                return matches;
            }*/
            return matches;
        }
        Err(_) => {
            println!("Invalid pattern");
            matches.push(String::from("loggyxp: invalid regex pattern"));
            return matches;
        }
    }
}

pub fn remove_duplicates(matches: &Vec<String>) -> Vec<String>
{
    let mut non_duplicates = matches.clone();

    non_duplicates.sort();
    non_duplicates.dedup();

    return non_duplicates;

}